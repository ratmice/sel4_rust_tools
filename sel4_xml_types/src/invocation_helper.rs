use crate::invocations::*;
pub (crate) trait AttributeExt {
    type Error;

    fn opt_attr(self, attr: &'_ str) -> Option<String>;
    fn req_attr(self, attr: &'_ str) -> Result<String, Self::Error>;
}

impl AttributeExt for roxmltree::Node<'_, '_> {
    type Error = InvocationGenError;
    fn opt_attr(self, attr: &'_ str) -> Option<String> {
        self.attribute(attr).map(str::to_string)
    }

    fn req_attr(self, attr: &'_ str) -> Result<String, InvocationGenError> {
        self.attribute(attr)
            .map(str::to_string)
            .ok_or_else(|| InvocationGenError::Attribute(attr.to_string(), file!(), line!()))
    }
}

pub(crate) fn skip_irrelevant<'a, 'b>(
    node: Option<roxmltree::Node<'a, 'b>>,
) -> Option<roxmltree::Node<'a, 'b>> {
    match node {
        None => None,
        Some(node)
            if (node.node_type() == roxmltree::NodeType::Text
                && node.text().unwrap().trim().is_empty())
                || node.node_type() == roxmltree::NodeType::Comment =>
        {
            node.next_sibling()
        }
        node => node,
    }
}

pub(crate) fn take_return_value_from_element<'a, 'b>(
    node: roxmltree::Node<'a, 'b>,
    v: &mut Vec<Return>,
) -> Result<Option<roxmltree::Node<'a, 'b>>, InvocationGenError> {
    match node.node_type() {
        roxmltree::NodeType::Element => match node.tag_name().name().to_lowercase() {
            s if s == "return" => {
                for child in node.children() {
                    let child = Return::try_from(child)?;
                    v.push(child);
                }
                Ok(node.next_sibling())
            }
            _ => Ok(Some(node)),
        },
        _ => Ok(Some(node)),
    }
}

pub(crate) fn take_leaves_from_element<'a, 'b>(
    node: roxmltree::Node<'a, 'b>,
    name: &'_ str,
    v: &mut Vec<DocLeaf>,
) -> Result<Option<roxmltree::Node<'a, 'b>>, InvocationGenError> {
    match node.node_type() {
        roxmltree::NodeType::Element => match node.tag_name().name().to_lowercase() {
            s if s == name => {
                for child in node.children() {
                    let child = DocLeaf::try_from(child)?;
                    v.push(child);
                }
                Ok(node.next_sibling())
            }
            _ => Ok(Some(node)),
        },
        _ => Ok(Some(node)),
    }
}

// It is kind of annoying but somehow I ended up with 2 different mechanisms
// for ignoring whitespace elements..
pub(crate) enum WhitespaceOr<'a, 'b, T> {
    Whitespace(roxmltree::Node<'a, 'b>),
    T(T),
}

impl<'a, 'b, T> TryFrom<roxmltree::Node<'a, 'b>> for WhitespaceOr<'a, 'b, T>
where
    T: TryFrom<roxmltree::Node<'a, 'b>, Error = InvocationGenError>,
{
    type Error = InvocationGenError;
    fn try_from(node: roxmltree::Node<'a, 'b>) -> Result<Self, Self::Error> {
        if node.is_text() && node.text().unwrap().trim() == "" {
            Ok(WhitespaceOr::Whitespace(node))
        } else {
            Ok(WhitespaceOr::T(T::try_from(node)?))
        }
    }
}

impl<'a, 'b, T> WhitespaceOr<'a, 'b, T>
where
    T: TryFrom<roxmltree::Node<'a, 'b>, Error = InvocationGenError>,
{
    #[allow(unused)]
    pub(crate) fn ignore_whitespace(self: WhitespaceOr<'a, 'b, T>) -> Option<T> {
        match self {
            WhitespaceOr::T(child) => Some(child),
            _ => None,
        }
    }
}
