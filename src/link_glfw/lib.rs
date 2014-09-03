// Copyright 2014 The GLFW-RS Developers. For a full listing of the authors,
// refer to the AUTHORS file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![crate_name = "link_glfw"]
#![feature(plugin_registrar, quote)]

#[cfg(not(target_os="android"))]
extern crate rustc;
#[cfg(not(target_os="android"))]
extern crate syntax;

#[cfg(not(target_os="android"))]
use std::gc::{Gc, GC};
#[cfg(not(target_os="android"))]
use std::io::Command;
#[cfg(not(target_os="android"))]
use std::str;
#[cfg(not(target_os="android"))]
use syntax::ast;
#[cfg(not(target_os="android"))]
use syntax::codemap;
#[cfg(not(target_os="android"))]
use syntax::ext::base;
#[cfg(not(target_os="android"))]
use syntax::ext::build::AstBuilder;
#[cfg(not(target_os="android"))]
use syntax::parse::token;
#[cfg(not(target_os="android"))]
use intern_str = syntax::parse::token::intern_and_get_ident;

#[cfg(not(target_os="android"))]
#[plugin_registrar]
pub fn registrar(reg: &mut rustc::plugin::Registry) {
    reg.register_syntax_extension(token::intern("link_glfw"),
                                  base::ItemModifier(expand));
}

#[cfg(not(target_os="android"))]
fn lit_str(s: token::InternedString) -> ast::Lit_ {
    ast::LitStr(s, ast::CookedStr)
}

#[cfg(not(target_os="android"))]
enum LinkKind {
    Unknown,
    Framework,
}

#[cfg(not(target_os="android"))]
fn attr_link(context: &mut base::ExtCtxt, span: codemap::Span,
             name: token::InternedString, kind: LinkKind) -> ast::Attribute {
    let mut meta_items = vec![
        context.meta_name_value(span, intern_str("name"), lit_str(name)),
    ];
    match kind {
        Framework => {
            meta_items.push(context.meta_name_value(
                span, intern_str("kind"),
                lit_str(intern_str("framework"))
            ));
        },
        _ => {},
    }
    let meta_list = context.meta_list(span, intern_str("link"), meta_items);
    context.attribute(span, meta_list)
}

#[cfg(not(target_os="android"))]
pub fn expand(context: &mut base::ExtCtxt, span: codemap::Span,
              _meta_item: Gc<ast::MetaItem>, item: Gc<ast::Item>
              ) -> Gc<ast::Item> {
    let out = Command::new("pkg-config")
        .arg("--static")
        .arg("--libs-only-l")
        .arg("--libs-only-other")
        .arg("--print-errors")
        .arg("glfw3")
        .output();
    match out {
        Ok(out) => {
            if out.status.success() {
                let mut item = (*item).clone();
                str::from_utf8(out.output.as_slice()).map(|output| {
                    let mut expect_framework = false;
                    for word in output.words() {
                        if word.starts_with("-l") {
                            item.attrs.push(attr_link(
                                context, span,
                                intern_str(word.slice_from(2)),
                                Unknown,
                            ));
                        } else if expect_framework {
                            expect_framework = false;
                            item.attrs.push(attr_link(
                                context, span,
                                intern_str(word),
                                Framework,
                            ));
                        } else if word.starts_with("-framework") {
                            expect_framework = true;
                        }
                    }
                });
                box (GC) item
            } else {
                context.span_err(
                    span,
                    format!(
                        "error returned by \
                        `pkg-config`: ({})\n\
                        `pkg-config stdout`: {}\n\
                        `pkg-config stderr`: {}",
                        out.status,
                        String::from_utf8(out.output).unwrap(),
                        String::from_utf8(out.error).unwrap())
                        .as_slice());
                item
            }
        },
        Err(e) => {
            context.span_err(span, format!("io error: {}", e).as_slice());
            item
        },
    }
}
