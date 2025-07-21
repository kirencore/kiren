use crate::typescript::*;

#[test]
fn test_import_export_handling() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
import { Component } from 'react';
export default class MyComponent extends Component {
    render() {
        return null;
    }
}
"#;

    let result = transpiler.transpile(typescript);
    
    // Should not panic with imports/exports
    assert!(result.is_ok());
}

#[test]
fn test_named_imports() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
import { useState, useEffect } from 'react';
import { Button, Input } from 'antd';
"#;

    let result = transpiler.transpile(typescript);
    assert!(result.is_ok());
}

#[test]
fn test_default_imports() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
import React from 'react';
import axios from 'axios';
"#;

    let result = transpiler.transpile(typescript);
    assert!(result.is_ok());
}

#[test]
fn test_namespace_imports() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
import * as React from 'react';
import * as fs from 'fs';
"#;

    let result = transpiler.transpile(typescript);
    assert!(result.is_ok());
}

#[test]
fn test_type_only_imports() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
import type { User } from './types';
import type { ComponentProps } from 'react';
"#;

    let result = transpiler.transpile(typescript);
    assert!(result.is_ok());
}

#[test]
fn test_mixed_imports() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
import React, { useState, type ComponentProps } from 'react';
import { Button, type ButtonProps } from 'antd';
"#;

    let result = transpiler.transpile(typescript);
    assert!(result.is_ok());
}

#[test]
fn test_export_statements() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
export const config = { version: '1.0.0' };
export function helper() { return 'help'; }
export class Utils {}
export type UserType = { name: string };
export interface IUser { name: string; }
"#;

    let result = transpiler.transpile(typescript);
    assert!(result.is_ok());
}

#[test]
fn test_re_exports() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
export { Button } from 'antd';
export type { User } from './types';
export * from './utils';
export * as helpers from './helpers';
"#;

    let result = transpiler.transpile(typescript);
    assert!(result.is_ok());
}