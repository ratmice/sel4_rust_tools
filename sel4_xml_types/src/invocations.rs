use roxmltree as xml;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::invocation_helper::*;

//
// While the syscall API is easily parsed with serde based xml libraries
//
// The xml defined in sel4_idl.dtd however doesn't easily fit into the
// serde data model. In particular the serde parsers don't deal well with
// elements such as: `<description> text <texttt text="foo"/> <description>`
//
// which would involve an `enum { Text(String), TextTT{text: String} }`
// as a child of description.
//
// Instead this uses [roxmltree](https://docs.rs/roxmltree/)
// It is unfortunate to use multiple xml parsers in this library,
// but it is easier to just use serde when it works well.
//
// For now we'll just depend upon multiple parsers, if a wrapper for roxmltree
// implements serde Deserialization this could use that instead.
//
// The current implementation does not check that sequences/children are emitted in the correct order.
// However this can be checked from the resulting Vec structures.
//

#[derive(Error, Debug)]
pub enum InvocationGenError {
    #[error("roxmltree error {0}")]
    Roxmltree(#[from] roxmltree::Error),
    #[error("attribute '{0}' not found")]
    Attribute(String, &'static str, u32),
    #[error("Cannot convert element from unknown element '{0} ")]
    UnsupportedNodeType(String, &'static str, u32),
}

/// While the types herein implement `Deserialize`
/// They would not Deserialize to equivalent XML.
/// 
/// This uses a read-only XML parser, and converts the parsed xml
/// into more (hopefully) convenient types.
#[derive(Debug, Serialize, Deserialize)]
pub struct Api {
    pub name: Option<String>,
    pub label_prefix: Option<String>,
    pub children: Vec<ApiElement>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ApiElement {
    StructElem {
        name: String,
        members: Vec<String>,
    },
    Interface {
        name: String,
        manual_name: Option<String>,
        cap_desc: Option<String>,
        methods: Vec<Method>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Method {
    // Attributes
    pub name: String,
    pub id: String,
    pub condition: Option<String>,
    pub manual_name: Option<String>,
    pub manual_label: Option<String>,
    pub brief: Vec<DocLeaf>,
    // Purely a child element
    pub description: Vec<DocLeaf>,
    pub return_value: Vec<Return>,
    pub cap_param: Option<CapParam>,
    pub params: Vec<Param>,
    pub errors: Vec<ErrorElement>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Return {
    ErrorEnumDesc,
    Leaves(DocLeaf),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Param {
    typ: String,
    name: String,
    dir: String,
    // From either an attribute, a child element
    // or (In a case which should perhaps be excluded)
    // if both attribute and a child <description>foo</description>
    // are set, this could contain both
    description: Vec<DocLeaf>,
    errors: Vec<ErrorElement>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorElement {
    name: String,
    // From either an attribute, a child element
    // or (In a case which should perhaps be excluded)
    // if both attribute and a child <description>foo</description>
    // are set, this could contain both
    description: Vec<DocLeaf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapParam {
    append_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DocLeaf {
    DocRef(Vec<LeafNode>),
    Leaf(LeafNode),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LeafNode {
    PCData(String),
    TextTT(String),
    AutoRef { label: String },
    ShortRef { sec: String },
    Obj { name: String },
}

impl<'a> TryFrom<roxmltree::Document<'a>> for Api {
    type Error = InvocationGenError;
    fn try_from(doc: xml::Document<'a>) -> Result<Api, Self::Error> {
        let element = doc.root_element();

        let name = element.attribute("name").map(str::to_string);
        let label_prefix = element.attribute("label_prefix").map(str::to_string);
        let mut children = Vec::new();
        for child in element.children() {
            let child = WhitespaceOr::<ApiElement>::try_from(child)?.ignore_whitespace();
            if let Some(child) = child {
                children.push(child)
            }
        }

        Ok(Api {
            name,
            label_prefix,
            children,
        })
    }
}


impl TryFrom<roxmltree::Node<'_, '_>> for Return {
    type Error = InvocationGenError;
    fn try_from(node: roxmltree::Node<'_, '_>) -> Result<Return, InvocationGenError> {
        match node.node_type() {
            roxmltree::NodeType::Element => match node.tag_name().name().to_lowercase() {
                s if s == "errorenumdesc" => Ok(Return::ErrorEnumDesc),
                _ => Ok(Return::Leaves(DocLeaf::try_from(node)?)),
            },
            _ => Ok(Return::Leaves(DocLeaf::try_from(node)?)),
        }
    }
}

impl TryFrom<roxmltree::Node<'_, '_>> for ErrorElement {
    type Error = InvocationGenError;
    fn try_from(node: roxmltree::Node<'_, '_>) -> Result<ErrorElement, InvocationGenError> {
        let name = node.req_attr("name")?;
        let description_attr = node.opt_attr("description");
        let mut description = Vec::new();

        if let Some(description_attr) = description_attr {
            description.push(DocLeaf::Leaf(LeafNode::PCData(description_attr)))
        }

        if let Some(node) = skip_irrelevant(node.first_child()) {
            let _node = take_leaves_from_element(node, "description", &mut description)?;
        } 

        Ok(ErrorElement {
            name,
            description,
        })
    }
}

impl TryFrom<roxmltree::Node<'_, '_>> for DocLeaf {
    type Error = InvocationGenError;
    fn try_from(node: roxmltree::Node<'_, '_>) -> Result<DocLeaf, InvocationGenError> {
        Ok(match node.node_type() {
            roxmltree::NodeType::Element if node.tag_name().name().to_lowercase() == "docref" => {
                let mut leaves = Vec::new();

                for leaf in node.children() {
                    let leaf = LeafNode::try_from(leaf)?;
                    leaves.push(leaf)
                }

                DocLeaf::DocRef(leaves)
            }
            _ => DocLeaf::Leaf(LeafNode::try_from(node)?),
        })
    }
}

impl TryFrom<roxmltree::Node<'_, '_>> for LeafNode {
    type Error = InvocationGenError;
    fn try_from(node: roxmltree::Node<'_, '_>) -> Result<LeafNode, Self::Error> {
        if node.is_text() {
            Ok(LeafNode::PCData(node.text().unwrap().to_string()))
        } else {
            let tag_name = node.tag_name().name().to_lowercase();
            match node.node_type() {
                roxmltree::NodeType::Element if tag_name == "texttt" => {
                    Ok(LeafNode::TextTT(node.req_attr("text")?))
                }
                roxmltree::NodeType::Element if tag_name == "shortref" => Ok(LeafNode::ShortRef {
                    sec: node.req_attr("sec")?,
                }),
                roxmltree::NodeType::Element if tag_name == "autoref" => Ok(LeafNode::ShortRef {
                    sec: node.req_attr("label")?,
                }),
                roxmltree::NodeType::Element if tag_name == "obj" => Ok(LeafNode::Obj {
                    name: node.req_attr("name")?,
                }),
                _ => Err(InvocationGenError::UnsupportedNodeType(
                    node.tag_name().name().to_string(),
                    file!(),
                    line!(),
                )),
            }
        }
    }
}

impl<'a, 'b> TryFrom<roxmltree::Node<'a, 'b>> for Method {
    type Error = InvocationGenError;
    fn try_from(node: roxmltree::Node<'a, 'b>) -> Result<Self, InvocationGenError> {
        // handle attributes
        let name = node.req_attr("name")?;
        let id = node.req_attr("id")?;
        let condition = node.opt_attr("condition");
        let manual_name = node.opt_attr("manual_name");
        let manual_label = node.opt_attr("manual_label");

        // handle children
        let mut brief = Vec::new();
        let node = node.first_child();
        let node = if let Some(node) = skip_irrelevant(node) {
            take_leaves_from_element(node, "brief", &mut brief)?
        } else {
            None
        };

        let mut description = Vec::new();
        let node = if let Some(node) = skip_irrelevant(node) {
            take_leaves_from_element(node, "description", &mut description)?
        } else {
            None
        };

        let mut return_value = Vec::new();
        let node = if let Some(node) = skip_irrelevant(node) {
            take_return_value_from_element(node, &mut return_value)?
        } else {
            None
        };

        let mut cap_param = None;
        let node = if let Some(node) = skip_irrelevant(node) {
            match node.node_type() {
                roxmltree::NodeType::Element
                    if node.tag_name().name().to_lowercase() == "cap_param" =>
                {
                    cap_param = node.opt_attr("append_description").map(|text| CapParam {
                        append_description: text,
                    });
                    node.next_sibling()
                }
                _ => Some(node),
            }
        } else {
            None
        };

        let mut params = Vec::new();
        let mut node = node;
        while let Some(a_node) = skip_irrelevant(node) {
            if a_node.node_type() == roxmltree::NodeType::Element
                && a_node.tag_name().name().to_lowercase() == "param"
            {
                params.push(Param::try_from(a_node)?);
                node = a_node.next_sibling();
            } else {
                break;
            }
        }
        let errors = Vec::new();

        Ok(Method {
            name,
            id,
            condition,
            manual_name,
            manual_label,
            brief,
            description,
            return_value,
            cap_param,
            params,
            errors,
        })
    }
}

impl<'a, 'b> TryFrom<roxmltree::Node<'a, 'b>> for Param {
    type Error = InvocationGenError;
    fn try_from(node: roxmltree::Node<'a, 'b>) -> Result<Self, InvocationGenError> {
        match node.node_type() {
            xml::NodeType::Element => match node.tag_name().name().to_lowercase() {
                s if s == "param" => {
                    let typ = node.req_attr("type")?;
                    let name = node.req_attr("name")?;
                    let dir = node.req_attr("dir")?;
                    let description_attr = node.opt_attr("description");
                    let node = node.first_child();
                    let mut description = Vec::new();
                    if let Some(description_attr) = description_attr {
                        description.push(DocLeaf::Leaf(LeafNode::PCData(description_attr)));
                    }

                    let mut node = if let Some(node) = skip_irrelevant(node) {
                        take_leaves_from_element(node, "description", &mut description)?
                    } else {
                        None
                    };
                    let mut errors = Vec::new();
                    while let Some(child) = skip_irrelevant(node) {
                        let error = ErrorElement::try_from(child)?;
                        errors.push(error);
                        node = child.next_sibling();
                    }

                    Ok(Param {
                        typ,
                        name,
                        dir,                        
                        description,
                        errors,
                    })
                }
                _ => Err(InvocationGenError::UnsupportedNodeType(
                    node.tag_name().name().to_string(),
                    file!(),
                    line!(),
                )),
            },
            _ => Err(InvocationGenError::UnsupportedNodeType(
                node.tag_name().name().to_string(),
                file!(),
                line!(),
            )),
        }
    }
}

impl<'a, 'b> TryFrom<roxmltree::Node<'a, 'b>> for ApiElement {
    type Error = InvocationGenError;
    fn try_from(node: roxmltree::Node<'a, 'b>) -> Result<Self, Self::Error> {
        match node.tag_name().name().to_lowercase() {
            s if s == "struct" => {
                let name = node.req_attr("name")?;
                let mut members = Vec::new();

                for child in node.children() {
                    if let roxmltree::NodeType::Element = child.node_type() {
                        let child_name = child.req_attr("name")?;
                        members.push(child_name);
                    }
                }

                Ok(ApiElement::StructElem { name, members })
            }

            s if s == "interface" => {
                let name = node.req_attr("name")?;
                let manual_name = node.opt_attr("manual_name");
                let cap_desc = node.opt_attr("capability_description");
                let mut methods = Vec::new();
                for child in node.children() {
                    let child = WhitespaceOr::<Method>::try_from(child)?.ignore_whitespace();
                    if let Some(child) = child {
                        methods.push(child);
                    }
                }

                Ok(ApiElement::Interface {
                    name,
                    manual_name,
                    cap_desc,
                    methods,
                })
            }

            s => Err(InvocationGenError::UnsupportedNodeType(s, file!(), line!())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;
    use std::fs::File;
    use std::io::{BufReader, Read};

    #[test]
    fn test() -> Result<(), TestError> {
        let files = [
            "../../../kernel/libsel4/arch_include/x86/interfaces/sel4arch.xml",
            "../../../kernel/libsel4/arch_include/arm/interfaces/sel4arch.xml",
            "../../../kernel/libsel4/arch_include/riscv/interfaces/sel4arch.xml",
            "../../../kernel/libsel4/include/interfaces/sel4.xml",
            "../../../kernel/libsel4/sel4_arch_include/aarch32/interfaces/sel4arch.xml",
            "../../../kernel/libsel4/sel4_arch_include/aarch64/interfaces/sel4arch.xml",
            "../../../kernel/libsel4/sel4_arch_include/ia32/interfaces/sel4arch.xml",
            "../../../kernel/libsel4/sel4_arch_include/riscv32/interfaces/sel4arch.xml",
            "../../../kernel/libsel4/sel4_arch_include/riscv64/interfaces/sel4arch.xml",
            "../../../kernel/libsel4/sel4_arch_include/x86_64/interfaces/sel4arch.xml",
        ];

        for filename in files {
            let f = File::open(filename)?;
            let mut reader = BufReader::new(f);
            let mut s = String::new();
            let _len = reader.read_to_string(&mut s)?;
            let tree = roxmltree::Document::parse(&s)?;
            let api: Api = Api::try_from(tree)?;
            println!("{:#?}", api)
        }
        Ok(())
    }
}
