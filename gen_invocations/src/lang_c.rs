// Templates taken from sel4 invocation_header_gen.py
//# Copyright 2020, Data61, CSIRO (ABN 41 687 119 230)
//#
//# SPDX-License-Identifier: BSD-2-Clause or GPL-2.0-only
//#

const COMMON_HEADER: &str = r#"
/*
 * Copyright 2020, Data61, CSIRO (ABN 41 687 119 230)
 *
{%- if libsel4 -%}
 * SPDX-License-Identifier: BSD-2-Clause
{%- else -%}
 * SPDX-License-Identifier: GPL-2.0-only
{%- endif %}
 */

/* This header was generated by kernel/tools/invocation_header_gen.py.
 *
 * To add an invocation call number, edit libsel4/include/interfaces/sel4.xml.
 *
 */"#;

pub const INVOCATION_TEMPLATE: &str = const_format::concatcp!(
    COMMON_HEADER,
    r#"
#ifndef __{{header_title}}_INVOCATION_H
#define __{{header_title}}_INVOCATION_H

enum invocation_label {
    InvalidInvocation,
    {%- for label, condition in invocations %}
    {%- if condition %}
#if {{condition}}
    {%- endif %}
    {{label}},
    {%- if condition %}
#endif
    {%- endif %}
    {%- endfor %}
    nInvocationLabels
};

{%- if libsel4 %}
#include <sel4/sel4_arch/invocation.h>
#include <sel4/arch/invocation.h>
{%- endif %}

#endif /* __{{header_title}}_INVOCATION_H */

"#
);

pub const SEL4_ARCH_INVOCATION_TEMPLATE: &str = const_format::concatcp!(
    COMMON_HEADER,
    r#"
#ifndef __{{header_title}}_SEL4_ARCH_INVOCATION_H
#define __{{header_title}}_SEL4_ARCH_INVOCATION_H

{%- if not libsel4 %}
#include <api/invocation.h>
{%- endif %}

enum sel4_arch_invocation_label {
    {%- for label, condition in invocations %}
        {%- if condition %}
            {%- if loop.first %}
#error "First sel4_arch invocation label cannot be conditional"
            {%- endif %}
#if {{condition}}
        {%- endif %}
        {%- if loop.first %}
    {{label}} = nInvocationLabels,
        {%- else %}
    {{label}},
        {%- endif %}
        {%- if condition %}
#endif
        {%- endif %}
    {%- endfor %}
    {%- if invocations|length == 0 %}
    nSeL4ArchInvocationLabels = nInvocationLabels
    {%- else %}
    nSeL4ArchInvocationLabels
    {%- endif %}
};

#endif /* __{{header_title}}_SEL4_ARCH_INVOCATION_H */

"#
);

pub const ARCH_INVOCATION_TEMPLATE: &str = const_format::concatcp!(
    COMMON_HEADER,
    r#"
#ifndef __{{header_title}}_ARCH_INVOCATION_H
#define __{{header_title}}_ARCH_INVOCATION_H

{%- if not libsel4 %}
#include <arch/api/sel4_invocation.h>
{%- endif %}

enum arch_invocation_label {
    {%- for label, condition in invocations %}
    {%- if condition %}
    {%- if loop.first  %}
#error "First arch invocation label cannot be conditional"
    {%- endif %}
#if {{condition}}
    {%- endif %}
    {%- if loop.first %}
    {{label}} = nSeL4ArchInvocationLabels,
    {%- else %}
    {{label}},
    {%- endif %}
    {%- if condition %}
#endif
    {%- endif %}
    {%- endfor %}
    {%- if invocations|length == 0 %}
    nArchInvocationLabels = nSeL4ArchInvocationLabels
    {%- else %}
    nArchInvocationLabels
    {%- endif %}
};

#endif /* __{{header_title}}_ARCH_INVOCATION_H */

"#
);
