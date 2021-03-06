const COMMON_HEADER: &str = r#"
/* This header was generated by kernel/tools/syscall_header_gen.py.
 *
 * To add a system call number, edit kernel/include/api/syscall.xml
 *
 */"#;

pub const KERNEL_HEADER_TEMPLATE: &str = const_format::concatcp!(
    r#"/*
* Copyright 2020, Data61, CSIRO (ABN 41 687 119 230)
*
* SPDX-License-Identifier: GPL-2.0-only
*/

"#,
    const_format::concatcp!(
        COMMON_HEADER,
        r#"
#pragma once

#ifdef __ASSEMBLER__

/* System Calls */
{%- for condition, list in assembler  -%}
    {%- for syscall, syscall_number in list %}
#define SYSCALL_{{upper(syscall)}} ({{syscall_number}})
    {%- endfor  %}
{%- endfor  %}

#endif /* __ASSEMBLER__ */

#define SYSCALL_MAX (-1)
#define SYSCALL_MIN ({{syscall_min}})

#ifndef __ASSEMBLER__

enum syscall {
{%- for condition, list in enum %}
   {%- if condition | length > 0 %}
#if {{condition}}
   {%- endif %}
   {%- for syscall, syscall_number in list %}
    Sys{{syscall}} = {{syscall_number}},
   {%- endfor %}
   {%- if condition | length > 0 %}
#endif /* {{condition}} */
   {%- endif %}
{%- endfor %}
};
typedef word_t syscall_t;

/* System call names */
#ifdef CONFIG_DEBUG_BUILD
static char *syscall_names[] UNUSED = {
{%- for condition, list in assembler  -%}
    {%- for syscall, syscall_number in list %}
         [{{syscall_number * -1}}] = "{{syscall}}",
    {%- endfor %}
{%- endfor %}
};
#endif /* CONFIG_DEBUG_BUILD */
#endif /* !__ASSEMBLER__ */

"#
    )
);

pub const LIBSEL4_HEADER_TEMPLATE: &str = const_format::concatcp!(
    r#"/*
* Copyright 2020, Data61, CSIRO (ABN 41 687 119 230)
*
* SPDX-License-Identifier: BSD-2-Clause
*/

"#,
    const_format::concatcp!(
        COMMON_HEADER,
        r#"
#pragma once

#include <autoconf.h>

typedef enum {
{%- for condition, list in enum %}
   {%- if condition | length > 0 %}
#if {{condition}}
   {%- endif %}
   {%- for syscall, syscall_number in list %}
       seL4_Sys{{syscall}} = {{syscall_number}},
   {%- endfor %}
   {%- if condition | length > 0 %}
#endif /* {{condition}} */
   {%- endif %}
{%- endfor %}
    SEL4_FORCE_LONG_ENUM(seL4_Syscall_ID)
} seL4_Syscall_ID;

"#
    )
);
