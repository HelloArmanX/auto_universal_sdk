use arboard::Clipboard;
use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, text, text_editor, text_input,
};
use iced::{Element, Font, Length, Settings, Theme};

fn main() -> iced::Result {
    iced::application(
        "Rust 代码生成器",
        CodeGenerator::update,
        CodeGenerator::view,
    )
    .settings(Settings {
        default_font: Font::with_name("PingFang SC"),
        ..Default::default()
    })
    .run()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OperationType {
    Database,
    Network,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::Database => write!(f, "数据库操作"),
            OperationType::Network => write!(f, "网络请求"),
        }
    }
}

impl OperationType {
    const ALL: [OperationType; 2] = [OperationType::Database, OperationType::Network];
}

struct CodeGenerator {
    project_path: String,
    function_name: String,
    function_params: String,
    callback_return_type: String,
    request_body_name: String,
    request_file_name: String,
    operation_type: Option<OperationType>,
    pass_params_to_request: bool,
    generate_db_functions: bool,
    engine_sync_content: text_editor::Content,
    engine_async_content: text_editor::Content,
    module_content: text_editor::Content,
    request_builder_content: text_editor::Content,
    request_struct_content: text_editor::Content,
    test_method_content: text_editor::Content,
    db_agent_content: text_editor::Content,
    db_worker_content: text_editor::Content,
    db_sqlite_content: text_editor::Content,
    status_message: String,
}

#[derive(Debug, Clone)]
enum Message {
    ProjectPathChanged(String),
    FunctionNameChanged(String),
    FunctionParamsChanged(String),
    CallbackReturnTypeChanged(String),
    RequestBodyNameChanged(String),
    RequestFileNameChanged(String),
    OperationTypeSelected(OperationType),
    TogglePassParamsToRequest(bool),
    ToggleGenerateDbFunctions(bool),
    GenerateCode,
    ClearAll,
    CopyEngineSyncToClipboard,
    CopyEngineAsyncToClipboard,
    CopyModuleToClipboard,
    CopyRequestBuilderToClipboard,
    CopyRequestStructToClipboard,
    CopyTestMethodToClipboard,
    CopyDbAgentToClipboard,
    CopyDbWorkerToClipboard,
    CopyDbSqliteToClipboard,
    EngineSyncAction(text_editor::Action),
    EngineAsyncAction(text_editor::Action),
    ModuleAction(text_editor::Action),
    RequestBuilderAction(text_editor::Action),
    RequestStructAction(text_editor::Action),
    TestMethodAction(text_editor::Action),
    DbAgentAction(text_editor::Action),
    DbWorkerAction(text_editor::Action),
    DbSqliteAction(text_editor::Action),
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self {
            project_path: "/Users/dxd/workspace/gitlab2/Rust/JQK-rust-universal-imsdk".to_string(),
            function_name: String::new(),
            function_params: String::new(),
            callback_return_type: String::new(),
            request_body_name: String::new(),
            request_file_name: String::new(),
            operation_type: Some(OperationType::Network),
            pass_params_to_request: false,
            generate_db_functions: false,
            engine_sync_content: text_editor::Content::new(),
            engine_async_content: text_editor::Content::new(),
            module_content: text_editor::Content::new(),
            request_builder_content: text_editor::Content::new(),
            request_struct_content: text_editor::Content::new(),
            test_method_content: text_editor::Content::new(),
            db_agent_content: text_editor::Content::new(),
            db_worker_content: text_editor::Content::new(),
            db_sqlite_content: text_editor::Content::new(),
            status_message: String::new(),
        }
    }
}

impl CodeGenerator {
    fn update(&mut self, message: Message) {
        match message {
            Message::ProjectPathChanged(path) => {
                self.project_path = path;
            }
            Message::FunctionNameChanged(name) => {
                self.function_name = name;
            }
            Message::FunctionParamsChanged(params) => {
                // 尝试将Java风格参数转换为Rust风格
                // 如果输入看起来像Java风格（包含final或以逗号分隔的类型 变量名格式），则转换
                if params.contains("final ")
                    || params.split(',').any(|p| {
                        let trimmed = p.trim();
                        let parts: Vec<&str> = trimmed.split_whitespace().collect();
                        // 如果格式是 "类型 变量名" 且不包含冒号，则认为是Java风格
                        parts.len() >= 2 && !trimmed.contains(':')
                    })
                {
                    self.function_params = convert_java_params_to_rust(&params);
                } else {
                    self.function_params = params;
                }
            }
            Message::CallbackReturnTypeChanged(return_type) => {
                self.callback_return_type = return_type;
            }
            Message::RequestBodyNameChanged(name) => {
                self.request_body_name = name.clone();
                // 自动生成对应的 rust 文件名（snake_case）
                self.request_file_name = pascal_to_snake_case(&name);
            }
            Message::RequestFileNameChanged(name) => {
                self.request_file_name = name;
            }
            Message::OperationTypeSelected(op_type) => {
                self.operation_type = Some(op_type);
            }
            Message::TogglePassParamsToRequest(enabled) => {
                self.pass_params_to_request = enabled;
            }
            Message::ToggleGenerateDbFunctions(enabled) => {
                self.generate_db_functions = enabled;
            }
            Message::GenerateCode => {
                if self.function_name.is_empty() {
                    self.status_message = "错误：函数名称不能为空！".to_string();
                    return;
                }
                if self.function_params.is_empty() {
                    self.status_message = "错误：函数参数不能为空！".to_string();
                    return;
                }

                let rust_function_name = java_to_rust_naming(&self.function_name);

                // 生成各个部分的代码
                let engine_sync_code = self.generate_engine_sync_function(&rust_function_name);
                let engine_async_code = self.generate_engine_async_function(&rust_function_name);
                let module_code = self.generate_module_function(&rust_function_name);

                // 生成 request_builder 代码（仅网络请求模式）
                let request_builder_code = if self.operation_type == Some(OperationType::Network) {
                    self.generate_request_builder_function(&rust_function_name)
                } else {
                    String::new()
                };

                let request_struct_code = if !self.request_body_name.is_empty() {
                    self.generate_request_struct()
                } else {
                    String::new()
                };
                let test_method_code = self.generate_test_method(&rust_function_name);

                // 生成数据库函数代码
                let (db_agent_code, db_worker_code, db_sqlite_code) = if self.generate_db_functions
                {
                    (
                        self.generate_db_agent_function(&rust_function_name),
                        self.generate_db_worker_function(&rust_function_name),
                        self.generate_db_sqlite_function(&rust_function_name),
                    )
                } else {
                    (String::new(), String::new(), String::new())
                };

                self.engine_sync_content = text_editor::Content::with_text(&engine_sync_code);
                self.engine_async_content = text_editor::Content::with_text(&engine_async_code);
                self.module_content = text_editor::Content::with_text(&module_code);
                self.request_builder_content =
                    text_editor::Content::with_text(&request_builder_code);
                self.request_struct_content = text_editor::Content::with_text(&request_struct_code);
                self.test_method_content = text_editor::Content::with_text(&test_method_code);
                self.db_agent_content = text_editor::Content::with_text(&db_agent_code);
                self.db_worker_content = text_editor::Content::with_text(&db_worker_code);
                self.db_sqlite_content = text_editor::Content::with_text(&db_sqlite_code);

                self.status_message = "代码生成成功！".to_string();
            }
            Message::ClearAll => {
                // 不清空项目路径，只清空其他输入框
                self.function_name.clear();
                self.function_params.clear();
                self.callback_return_type.clear();
                self.request_body_name.clear();
                self.request_file_name.clear();
                self.operation_type = Some(OperationType::Network);
                self.engine_sync_content = text_editor::Content::new();
                self.engine_async_content = text_editor::Content::new();
                self.module_content = text_editor::Content::new();
                self.request_builder_content = text_editor::Content::new();
                self.request_struct_content = text_editor::Content::new();
                self.test_method_content = text_editor::Content::new();
                self.db_agent_content = text_editor::Content::new();
                self.db_worker_content = text_editor::Content::new();
                self.db_sqlite_content = text_editor::Content::new();
                self.status_message = "已清空所有输入！".to_string();
            }
            Message::CopyEngineSyncToClipboard => {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard.set_text(&self.engine_sync_content.text()).is_ok() {
                        self.status_message = "engine_sync.rs 已复制到剪贴板！".to_string();
                    } else {
                        self.status_message = "复制失败！".to_string();
                    }
                }
            }
            Message::CopyEngineAsyncToClipboard => {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard
                        .set_text(&self.engine_async_content.text())
                        .is_ok()
                    {
                        self.status_message = "engine_async.rs 已复制到剪贴板！".to_string();
                    } else {
                        self.status_message = "复制失败！".to_string();
                    }
                }
            }
            Message::CopyModuleToClipboard => {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard.set_text(&self.module_content.text()).is_ok() {
                        self.status_message = "module 文件已复制到剪贴板！".to_string();
                    } else {
                        self.status_message = "复制失败！".to_string();
                    }
                }
            }
            Message::CopyRequestBuilderToClipboard => {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard
                        .set_text(&self.request_builder_content.text())
                        .is_ok()
                    {
                        self.status_message = "request_builder 文件已复制到剪贴板！".to_string();
                    } else {
                        self.status_message = "复制失败！".to_string();
                    }
                }
            }
            Message::CopyRequestStructToClipboard => {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard
                        .set_text(&self.request_struct_content.text())
                        .is_ok()
                    {
                        self.status_message = "请求体结构已复制到剪贴板！".to_string();
                    } else {
                        self.status_message = "复制失败！".to_string();
                    }
                }
            }
            Message::EngineSyncAction(action) => {
                self.engine_sync_content.perform(action);
            }
            Message::EngineAsyncAction(action) => {
                self.engine_async_content.perform(action);
            }
            Message::ModuleAction(action) => {
                self.module_content.perform(action);
            }
            Message::RequestBuilderAction(action) => {
                self.request_builder_content.perform(action);
            }
            Message::RequestStructAction(action) => {
                self.request_struct_content.perform(action);
            }
            Message::CopyTestMethodToClipboard => {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard.set_text(&self.test_method_content.text()).is_ok() {
                        self.status_message = "测试方法已复制到剪贴板！".to_string();
                    } else {
                        self.status_message = "复制失败！".to_string();
                    }
                }
            }
            Message::TestMethodAction(action) => {
                self.test_method_content.perform(action);
            }
            Message::CopyDbAgentToClipboard => {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard.set_text(&self.db_agent_content.text()).is_ok() {
                        self.status_message = "db_agent.rs 已复制到剪贴板！".to_string();
                    } else {
                        self.status_message = "复制失败！".to_string();
                    }
                }
            }
            Message::CopyDbWorkerToClipboard => {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard.set_text(&self.db_worker_content.text()).is_ok() {
                        self.status_message = "db_worker.rs 已复制到剪贴板！".to_string();
                    } else {
                        self.status_message = "复制失败！".to_string();
                    }
                }
            }
            Message::CopyDbSqliteToClipboard => {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard.set_text(&self.db_sqlite_content.text()).is_ok() {
                        self.status_message = "db_sqlite.rs 已复制到剪贴板！".to_string();
                    } else {
                        self.status_message = "复制失败！".to_string();
                    }
                }
            }
            Message::DbAgentAction(action) => {
                self.db_agent_content.perform(action);
            }
            Message::DbWorkerAction(action) => {
                self.db_worker_content.perform(action);
            }
            Message::DbSqliteAction(action) => {
                self.db_sqlite_content.perform(action);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let title = text("Rust 代码生成器").size(28);

        let project_path_input = column![
            text("项目路径:"),
            text_input("输入项目路径", &self.project_path)
                .on_input(Message::ProjectPathChanged)
                .padding(8)
                .width(Length::Fill),
        ]
        .spacing(5);

        let function_name_input = column![
            text("函数名称 (Java 风格):"),
            text_input(
                "例如: deleteUltraGroupMessagesForAllChannel",
                &self.function_name
            )
            .on_input(Message::FunctionNameChanged)
            .padding(8)
            .width(Length::Fill),
        ]
        .spacing(5);

        let function_params_input = column![
            text("函数参数:"),
            text_input(
                "例如: params: SearchLocalFriendParams",
                &self.function_params
            )
            .on_input(Message::FunctionParamsChanged)
            .padding(8)
            .width(Length::Fill),
        ]
        .spacing(5);

        let callback_return_input = column![
            text("Callback 返回值类型:"),
            text_input("例如: Vec<FriendInfo>", &self.callback_return_type)
                .on_input(Message::CallbackReturnTypeChanged)
                .padding(8)
                .width(Length::Fill),
        ]
        .spacing(5);

        let request_body_input = column![
            text("请求体名称 (可选):"),
            text_input(
                "例如: SetUltraGroupOperateStatusRequest",
                &self.request_body_name
            )
            .on_input(Message::RequestBodyNameChanged)
            .padding(8)
            .width(Length::Fill),
        ]
        .spacing(5);

        let operation_type_picker = column![
            text("操作类型:"),
            pick_list(
                &OperationType::ALL[..],
                self.operation_type.as_ref(),
                Message::OperationTypeSelected,
            )
            .padding(8)
            .width(200),
        ]
        .spacing(5);

        let params_to_request_checkbox =
            checkbox("参数传递到 Request 结构体", self.pass_params_to_request)
                .on_toggle(Message::TogglePassParamsToRequest);

        let generate_db_functions_checkbox = checkbox("生成数据库函数", self.generate_db_functions)
            .on_toggle(Message::ToggleGenerateDbFunctions);

        let generate_button = button(text("生成代码").size(16))
            .on_press(Message::GenerateCode)
            .padding(10)
            .width(150);

        let clear_button = button(text("清空").size(16))
            .on_press(Message::ClearAll)
            .padding(10)
            .width(100);

        let status_color = if self.status_message.contains("错误") {
            iced::Color::from_rgb(1.0, 0.3, 0.3)
        } else if self.status_message.contains("成功")
            || self.status_message.contains("复制")
            || self.status_message.contains("清空")
        {
            iced::Color::from_rgb(0.3, 1.0, 0.3)
        } else {
            iced::Color::WHITE
        };

        let status = text(&self.status_message)
            .size(14)
            .style(move |_theme: &Theme| text::Style {
                color: Some(status_color),
            });

        // engine_sync.rs 输出框
        let engine_sync_section = column![
            row![
                text("engine_sync.rs").size(16),
                button(text("复制").size(14))
                    .on_press(Message::CopyEngineSyncToClipboard)
                    .padding(5),
            ]
            .spacing(10),
            text_editor(&self.engine_sync_content)
                .on_action(Message::EngineSyncAction)
                .height(200),
        ]
        .spacing(5);

        // engine_async.rs 输出框
        let engine_async_section = column![
            row![
                text("engine_async.rs").size(16),
                button(text("复制").size(14))
                    .on_press(Message::CopyEngineAsyncToClipboard)
                    .padding(5),
            ]
            .spacing(10),
            text_editor(&self.engine_async_content)
                .on_action(Message::EngineAsyncAction)
                .height(200),
        ]
        .spacing(5);

        // module 文件输出框
        let module_section = column![
            row![
                text("module 文件").size(16),
                button(text("复制").size(14))
                    .on_press(Message::CopyModuleToClipboard)
                    .padding(5),
            ]
            .spacing(10),
            text_editor(&self.module_content)
                .on_action(Message::ModuleAction)
                .height(200),
        ]
        .spacing(5);

        // request_builder 文件输出框（仅在网络请求模式下显示）
        let request_builder_section = if self.operation_type == Some(OperationType::Network) {
            column![
                row![
                    text("request_builder 文件").size(16),
                    button(text("复制").size(14))
                        .on_press(Message::CopyRequestBuilderToClipboard)
                        .padding(5),
                ]
                .spacing(10),
                text_editor(&self.request_builder_content)
                    .on_action(Message::RequestBuilderAction)
                    .height(200),
            ]
            .spacing(5)
        } else {
            column![]
        };

        // 请求体结构输出框（仅在有请求体名称时显示）
        let request_struct_section = if !self.request_body_name.is_empty() {
            column![
                row![
                    text("请求体结构").size(16),
                    text_input("rust 文件名", &self.request_file_name)
                        .on_input(Message::RequestFileNameChanged)
                        .padding(5)
                        .width(400),
                    button(text("复制").size(14))
                        .on_press(Message::CopyRequestStructToClipboard)
                        .padding(5),
                ]
                .spacing(10),
                text_editor(&self.request_struct_content)
                    .on_action(Message::RequestStructAction)
                    .height(200),
            ]
            .spacing(5)
        } else {
            column![]
        };

        // 测试方法输出框
        let test_method_section = column![
            row![
                text("测试方法").size(16),
                button(text("复制").size(14))
                    .on_press(Message::CopyTestMethodToClipboard)
                    .padding(5),
            ]
            .spacing(10),
            text_editor(&self.test_method_content)
                .on_action(Message::TestMethodAction)
                .height(200),
        ]
        .spacing(5);

        // 数据库函数输出框（仅在勾选生成数据库函数时显示）
        let db_sections = if self.generate_db_functions {
            column![
                column![
                    row![
                        text("db_agent.rs (A函数)").size(16),
                        button(text("复制").size(14))
                            .on_press(Message::CopyDbAgentToClipboard)
                            .padding(5),
                    ]
                    .spacing(10),
                    text_editor(&self.db_agent_content)
                        .on_action(Message::DbAgentAction)
                        .height(200),
                ]
                .spacing(5),
                column![
                    row![
                        text("db_worker.rs (B函数)").size(16),
                        button(text("复制").size(14))
                            .on_press(Message::CopyDbWorkerToClipboard)
                            .padding(5),
                    ]
                    .spacing(10),
                    text_editor(&self.db_worker_content)
                        .on_action(Message::DbWorkerAction)
                        .height(200),
                ]
                .spacing(5),
                column![
                    row![
                        text("db_sqlite.rs (C函数)").size(16),
                        button(text("复制").size(14))
                            .on_press(Message::CopyDbSqliteToClipboard)
                            .padding(5),
                    ]
                    .spacing(10),
                    text_editor(&self.db_sqlite_content)
                        .on_action(Message::DbSqliteAction)
                        .height(200),
                ]
                .spacing(5),
            ]
        } else {
            column![]
        };

        let content = column![
            title,
            project_path_input,
            function_name_input,
            function_params_input,
            callback_return_input,
            request_body_input,
            operation_type_picker,
            params_to_request_checkbox,
            generate_db_functions_checkbox,
            row![generate_button, clear_button].spacing(10),
            status,
            engine_sync_section,
            engine_async_section,
            module_section,
            request_builder_section,
            request_struct_section,
            test_method_section,
            db_sections,
        ]
        .spacing(15)
        .padding(20)
        .width(Length::Fill);

        container(scrollable(content)).center_x(Length::Fill).into()
    }

    fn generate_engine_sync_function(&self, rust_function_name: &str) -> String {
        let cb_type = if self.callback_return_type.is_empty() {
            "()".to_string()
        } else {
            self.callback_return_type.clone()
        };

        let cleaned_params = self.clean_params(&self.function_params);
        let str_conversions = self.generate_str_to_string_conversions();

        match self.operation_type {
            Some(OperationType::Database) => {
                format!(
                    r#"pub fn {}<CB>(&self, {}, cb: CB)
where
    CB: FnOnce(Result<{}, EngineError>) + Send + 'static,
{{
    let engine = self.engine.clone();
    let cb = self.cb_pool_once(cb);
{}
    self.post(async move {{
        let ret = engine.{}({}).await;
        cb(ret);
    }});
}}"#,
                    rust_function_name,
                    cleaned_params,
                    cb_type,
                    str_conversions,
                    rust_function_name,
                    self.extract_param_names_with_ref()
                )
            }
            Some(OperationType::Network) => {
                format!(
                    r#"pub fn {}<CB>(&self, {}, cb: CB)
where
    CB: FnOnce(Result<{}, EngineError>) + Send + 'static,
{{
    let engine = self.engine.clone();
    let callback = self.cb_pool_once(cb);
{}
    self.post(async move {{
        engine.{}({}, callback).await;
    }});
}}"#,
                    rust_function_name,
                    cleaned_params,
                    cb_type,
                    str_conversions,
                    rust_function_name,
                    self.extract_param_names_with_ref()
                )
            }
            None => String::new(),
        }
    }

    fn generate_engine_async_function(&self, rust_function_name: &str) -> String {
        let cb_type = if self.callback_return_type.is_empty() {
            "()".to_string()
        } else {
            self.callback_return_type.clone()
        };

        let params_with_ref = self.add_ref_to_str_params();
        let param_names = self.extract_param_names();

        // 生成 match 表达式的 Ok 分支
        let ok_match_pattern = if cb_type == "()" {
            "Ok(()) => \"\".to_string()".to_string()
        } else {
            "Ok(_) => \"\".to_string()".to_string()
        };

        match self.operation_type {
            Some(OperationType::Network) => {
                format!(
                    r#"pub async fn {}<CB>(&self, {}, cb: CB)
where
    CB: FnOnce(Result<{}, EngineError>) + Send + 'static,
{{
    let trace_id = self.ctx.logger().generate_trace_id();
    trace_i_json!(self.ctx.logger(), "P-{}-T", trace_id);
    let logger = self.ctx.logger().clone();
    let cb = move |ret: Result<{}, EngineError>| {{
        let str = match &ret {{
            {},
            Err(e) => e.to_string(),
        }};
        trace_i_json!(logger, "P-{}-R", trace_id, "result", &str);
        cb(ret);
    }};
    bugtags::{}(&self.ctx, {}, cb).await;
}}"#,
                    rust_function_name,
                    params_with_ref,
                    cb_type,
                    rust_function_name,
                    cb_type,
                    ok_match_pattern,
                    rust_function_name,
                    rust_function_name,
                    param_names
                )
            }
            Some(OperationType::Database) => {
                format!(
                    r#"pub async fn {}(&self, {}) -> Result<{}, EngineError> {{
    let trace_id = self.ctx.logger().generate_trace_id();
    trace_i_json!(self.ctx.logger(), "P-{}-T", trace_id);
    let ret = bugtags::{}(&self.ctx, {}).await;
    let str = match &ret {{
        Ok(_) => "".to_string(),
        Err(e) => e.to_string(),
    }};
    trace_i_json!(self.ctx.logger(), "P-{}-R", trace_id, "result", str);
    ret
}}"#,
                    rust_function_name,
                    params_with_ref,
                    cb_type,
                    rust_function_name,
                    rust_function_name,
                    param_names,
                    rust_function_name
                )
            }
            None => String::new(),
        }
    }

    fn generate_module_function(&self, rust_function_name: &str) -> String {
        let cb_type = if self.callback_return_type.is_empty() {
            "()".to_string()
        } else {
            self.callback_return_type.clone()
        };

        let params_with_ref = self.add_ref_to_str_params();
        let param_names = self.extract_param_names();

        match self.operation_type {
            Some(OperationType::Network) => {
                // 始终传递所有参数给 build_xxx_request 方法
                let build_params = if param_names.is_empty() {
                    "cb".to_string()
                } else {
                    format!("{}, cb", param_names)
                };

                format!(
                    r#"pub(crate) async fn {}<CB>(
    ctx: &Arc<EngineContext>,
    {},
    cb: CB,
)
where
    CB: FnOnce(Result<{}, EngineError>) + Send + 'static,
{{
    let query = ctx
        .request_builder()
        .build_{}_request({});
    ctx.send_query(query).await;
}}"#,
                    rust_function_name, params_with_ref, cb_type, rust_function_name, build_params
                )
            }
            Some(OperationType::Database) => {
                format!(
                    r#"pub(crate) async fn {}(
    ctx: &Arc<EngineContext>,
    {},
) -> Result<{}, EngineError> {{
    ctx.db_agent()
        .{}({})
        .await
}}"#,
                    rust_function_name, params_with_ref, cb_type, rust_function_name, param_names
                )
            }
            None => String::new(),
        }
    }

    fn generate_request_builder_function(&self, rust_function_name: &str) -> String {
        let cb_type = if self.callback_return_type.is_empty() {
            "()".to_string()
        } else {
            self.callback_return_type.clone()
        };

        // 使用规范化的参数处理方法
        let params_with_ref = self.normalize_params_for_request_builder();

        // 如果没有请求体名称，返回空字符串
        if self.request_body_name.is_empty() {
            return String::new();
        }

        // 生成 Pb 结构体名称（添加 "Pb" 前缀）
        let pb_request_name = format!("Pb{}", self.request_body_name);

        // 请求体结构名称（不带 "Pb" 前缀）
        let request_name = &self.request_body_name;

        // 构建函数名：在 rust_function_name 前添加 "build_"
        let build_function_name = format!("build_{}_request", rust_function_name);

        format!(
            r#"pub(crate) fn {}<CB>(
    &self,
    {},
    cb: CB,
) -> RmtpQuery
where
    CB: FnOnce(Result<{}, EngineError>) + Send + 'static,
{{
    let mut pb_req = {}::new();
    let req = {}::new(pb_req, cb);
    self.build_query(req.get_method(), "", req.get_qos(), Box::new(req))
}}"#,
            build_function_name, params_with_ref, cb_type, pb_request_name, request_name
        )
    }

    // 根据参数类型规范化参数名称
    fn normalize_param_name(&self, param_name: &str, param_type: &str) -> String {
        // 如果类型是 ConversationType 或 DbConversationType，统一使用 conv_type
        if param_type == "ConversationType" || param_type == "DbConversationType" {
            "conv_type".to_string()
        } else {
            param_name.to_string()
        }
    }

    // 规范化参数，确保格式为 "name: type"
    fn normalize_params_for_request_builder(&self) -> String {
        self.clean_params(&self.function_params)
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }

                // 分割参数为名称和类型
                let parts: Vec<&str> = trimmed.split(':').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    return Some(trimmed.to_string());
                }

                let param_name = parts[0];
                let mut param_type = parts[1].trim_end_matches(',').trim();

                // 如果类型是 String，转换为 &str
                if param_type == "String" {
                    param_type = "&str";
                }

                // 规范化参数名称
                let normalized_name = self.normalize_param_name(param_name, param_type);

                // 返回正确格式: name: type
                Some(format!("{}: {}", normalized_name, param_type))
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn generate_request_struct(&self) -> String {
        let cb_type = if self.callback_return_type.is_empty() {
            "()".to_string()
        } else {
            self.callback_return_type.clone()
        };

        let pb_request_name = format!("Pb{}", self.request_body_name);

        // 根据开关状态决定是否生成额外的成员变量
        let (extra_fields, extra_new_params, extra_field_inits) = if self.pass_params_to_request {
            // 开关打开，生成额外的成员变量
            (
                self.generate_struct_fields(),
                self.generate_new_params(),
                self.generate_field_inits(),
            )
        } else {
            // 开关关闭，不生成额外的成员变量
            (String::new(), String::new(), String::new())
        };

        // 决定结构体成员和 new 方法的内容
        let struct_fields = if extra_fields.is_empty() {
            format!("    pb_req: {},\n    cb: CB,", pb_request_name)
        } else {
            format!(
                "    pb_req: {},\n    cb: CB,\n{}",
                pb_request_name, extra_fields
            )
        };

        let new_params = if extra_new_params.is_empty() {
            format!("pb_req: {}, cb: CB", pb_request_name)
        } else {
            format!("pb_req: {}, cb: CB, {}", pb_request_name, extra_new_params)
        };

        let field_init = if extra_field_inits.is_empty() {
            "Self { pb_req, cb }".to_string()
        } else {
            format!("Self {{ pb_req, cb, {} }}", extra_field_inits)
        };

        format!(
            r#"use crate::engine_context::EngineContext;
use crate::engine_def::{{EngineError}};
use crate::rmtp::request::request_trait::Request;
use crate::rmtp::rmtp_def::RmtpQos;
use async_trait::async_trait;
use protobuf::Message;
use rust_universal_logger::err;
use std::sync::Arc;

pub(crate) struct {}<CB>
where
    CB: FnOnce(Result<{}, EngineError>) + Send + 'static,
{{
{}
}}

impl<CB> {}<CB>
where
    CB: FnOnce(Result<{}, EngineError>) + Send + 'static,
{{
    pub(crate) fn new({}) -> Self {{
        {}
    }}
}}

#[async_trait]
impl<CB> Request for {}<CB>
where
    CB: FnOnce(Result<{}, EngineError>) + Send + 'static,
{{
    fn get_method(&self) -> String {{
        "".to_string()
    }}

    fn get_qos(&self) -> RmtpQos {{
        RmtpQos::QosAtLastOnce
    }}

    async fn deal_with_response(
        self: Box<Self>,
        ctx: &Arc<EngineContext>,
        code: EngineError,
        timestamp: i64,
        msg_uid: String,
        pb_data: Option<Vec<u8>>,
    ) {{
        if EngineError::Success != code {{
            (self.cb)(Err(code));
            return;
        }}

        let pb_data = match pb_data {{
            Some(pb_data) => pb_data,
            None => return (self.cb)(Err(err!(EngineError::NetDataParserFailed))),
        }};

        // if EngineError::Success == code {{
        //     (self.cb)(Ok(()));
        // }} else {{
        //     (self.cb)(Err(code));
        // }}
        
        // TODO: 解析响应数据
        // let ret = ...;
        // (self.cb)(Ok(ret));
    }}

    fn get_pb_data(&self) -> Vec<u8> {{
        self.pb_req.write_to_bytes().unwrap_or_default()
    }}
}}"#,
            self.request_body_name,
            cb_type,
            struct_fields,
            self.request_body_name,
            cb_type,
            new_params,
            field_init,
            self.request_body_name,
            cb_type
        )
    }

    fn generate_test_method(&self, rust_function_name: &str) -> String {
        let param_definitions = self.generate_test_param_definitions();
        let param_names = self.extract_param_names_only();

        match self.operation_type {
            Some(OperationType::Database) => {
                // 数据库操作测试：参考 integration_ultra_group.rs
                let param_section = if !param_definitions.is_empty() {
                    format!("{}\n        ", param_definitions)
                } else {
                    String::new()
                };

                format!(
                    r#"#[test]
fn {0}() {{
    SHARED_RUNTIME.block_on(async {{
        const ROOM_NAME: &str = "test_room";
        let server_api = ServerApi::new();
        if !server_api.is_chatroom_exist(ROOM_NAME).await {{
            server_api.create_chatroom(ROOM_NAME).await;
        }}
        TESTER_A.connect().await.unwrap();
        let engine = &TESTER_A.engine;
        let (tx, rx) = oneshot::channel();
        {1}let ret = engine.{0}({2}).await;

        println!("{0}: {{:?}}", ret);
        assert!(ret.is_ok());
        tx.send(()).unwrap();

        match rx.await {{
            Ok(_) => {{}}
            Err(e) => {{
                debug!("{0} err: {{:?}}", e);
                assert!(false);
            }}
        }}
    }});
}}"#,
                    rust_function_name, param_section, param_names
                )
            }
            Some(OperationType::Network) => {
                // 网络请求测试：参考 integration_black_list.rs
                let param_section = if !param_definitions.is_empty() {
                    format!("{}\n        ", param_definitions)
                } else {
                    String::new()
                };

                let call_code = if param_names.is_empty() {
                    format!(
                        r#"{1}engine
                .{0}(|ret| {{
                    println!("{0}: {{:?}}", ret);
                    assert!(ret.is_ok());
                    tx.send(()).unwrap();
                }})
                .await;"#,
                        rust_function_name, param_section
                    )
                } else {
                    format!(
                        r#"{2}engine
                .{0}({1}, |ret| {{
                    println!("{0}: {{:?}}", ret);
                    assert!(ret.is_ok());
                    tx.send(()).unwrap();
                }})
                .await;"#,
                        rust_function_name, param_names, param_section
                    )
                };

                format!(
                    r#"#[test]
fn {0}() {{
    SHARED_RUNTIME.block_on(async {{
        const ROOM_NAME: &str = "test_room";
        let server_api = ServerApi::new();
        if !server_api.is_chatroom_exist(ROOM_NAME).await {{
            server_api.create_chatroom(ROOM_NAME).await;
        }}
        TESTER_A.connect().await.unwrap();
        let engine = &TESTER_A.engine;
        let (tx, rx) = oneshot::channel();
        {1}

        match rx.await {{
            Ok(_) => {{}}
            Err(e) => {{
                debug!("{0} err: {{:?}}", e);
                assert!(false);
            }}
        }}
    }});
}}"#,
                    rust_function_name, call_code
                )
            }
            None => String::new(),
        }
    }

    fn generate_struct_fields(&self) -> String {
        let cleaned_params = self.clean_params(&self.function_params);
        if cleaned_params.is_empty() {
            return String::new();
        }

        cleaned_params
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }

                let parts: Vec<&str> = trimmed.split(':').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    return None;
                }

                let param_name = parts[0];
                let mut param_type = parts[1];

                // 如果是 &str，转换为 String
                if param_type == "&str" {
                    param_type = "String";
                }

                // 规范化参数名称
                let normalized_name = self.normalize_param_name(param_name, param_type);

                Some(format!("    {}: {},", normalized_name, param_type))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_new_params(&self) -> String {
        let cleaned_params = self.clean_params(&self.function_params);
        if cleaned_params.is_empty() {
            return String::new();
        }

        cleaned_params
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }
                
                // 分割参数为名称和类型
                let parts: Vec<&str> = trimmed.split(':').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    return Some(trimmed.to_string());
                }
                
                let param_name = parts[0];
                let param_type = parts[1];
                
                // 规范化参数名称
                let normalized_name = self.normalize_param_name(param_name, param_type);
                
                Some(format!("{}: {}", normalized_name, param_type))
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn generate_field_inits(&self) -> String {
        let cleaned_params = self.clean_params(&self.function_params);
        if cleaned_params.is_empty() {
            return String::new();
        }

        cleaned_params
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }

                let parts: Vec<&str> = trimmed.split(':').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    return None;
                }

                let param_name = parts[0];
                let param_type = parts[1];

                // 规范化参数名称
                let normalized_name = self.normalize_param_name(param_name, param_type);

                // 如果参数是 &str，需要转换为 String
                if param_type == "&str" {
                    Some(format!("{}: {}.to_string()", normalized_name, normalized_name))
                } else {
                    Some(format!("{}", normalized_name))
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn extract_param_names(&self) -> String {
        self.clean_params(&self.function_params)
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }
                
                // 分割参数为名称和类型
                let parts: Vec<&str> = trimmed.split(':').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    return trimmed.split(':').next().map(|name| name.trim().to_string());
                }
                
                let param_name = parts[0];
                let param_type = parts[1].trim();
                
                // 规范化参数名称
                let normalized_name = self.normalize_param_name(param_name, param_type);
                
                Some(normalized_name)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn clean_params(&self, params: &str) -> String {
        // 去除末尾的逗号、空格等
        let cleaned = params.trim().trim_end_matches(',').trim().to_string();

        // 去除 cb: CB 参数
        let parts: Vec<&str> = cleaned.split(',').collect();
        let filtered_parts: Vec<&str> = parts
            .into_iter()
            .filter(|param| {
                let trimmed = param.trim();
                !trimmed.starts_with("cb:") && !trimmed.starts_with("cb :")
            })
            .collect();

        filtered_parts.join(", ")
    }

    fn extract_param_names_for_call(&self) -> String {
        self.clean_params(&self.function_params)
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }
                let name = trimmed.split(':').next()?.trim();

                // 如果参数名包含 &，说明已经是引用了
                if trimmed.contains("&str") {
                    Some(name.to_string())
                } else {
                    Some(name.to_string())
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn add_ref_to_str_params(&self) -> String {
        self.clean_params(&self.function_params)
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }
                
                // 分割参数为名称和类型
                let parts: Vec<&str> = trimmed.split(':').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    return Some(trimmed.to_string());
                }
                
                let param_name = parts[0];
                let mut param_type = parts[1].trim();
                
                // 如果类型是 String，转换为 &str
                if param_type == "String" {
                    param_type = "&str";
                }
                
                // 规范化参数名称
                let normalized_name = self.normalize_param_name(param_name, param_type);
                
                Some(format!("{}: {}", normalized_name, param_type))
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn generate_trace_params(&self) -> String {
        self.clean_params(&self.function_params)
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }
                trimmed.split(':').next().map(|name| {
                    let name = name.trim();
                    format!("\"{}\": {}", name, name)
                })
            })
            .collect::<Vec<_>>()
            .join(",\n            ")
    }

    fn generate_str_to_string_conversions(&self) -> String {
        let cleaned_params = self.clean_params(&self.function_params);
        let conversions: Vec<String> = cleaned_params
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }

                // 检查参数类型是否为 &str
                if trimmed.contains(": &str") {
                    let param_name = trimmed.split(':').next()?.trim();
                    Some(format!(
                        "    let {} = {}.to_string();",
                        param_name, param_name
                    ))
                } else {
                    None
                }
            })
            .collect();

        if conversions.is_empty() {
            String::new()
        } else {
            conversions.join("\n") + "\n"
        }
    }

    fn extract_param_names_with_ref(&self) -> String {
        self.clean_params(&self.function_params)
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }

                let param_name = trimmed.split(':').next()?.trim();

                // 如果参数类型是 &str，在调用时需要加 &
                if trimmed.contains(": &str") {
                    Some(format!("&{}", param_name))
                } else {
                    Some(param_name.to_string())
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn extract_param_names_only(&self) -> String {
        self.clean_params(&self.function_params)
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }

                // 分割参数为名称和类型
                let parts: Vec<&str> = trimmed.split(':').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    return trimmed.split(':').next().map(|name| name.trim().to_string());
                }
                
                let param_name = parts[0];
                let param_type = parts[1];
                
                // 规范化参数名称
                let normalized_name = self.normalize_param_name(param_name, param_type);
                Some(normalized_name)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn generate_test_param_definitions(&self) -> String {
        let cleaned_params = self.clean_params(&self.function_params);
        if cleaned_params.is_empty() {
            return String::new();
        }

        let definitions: Vec<String> = cleaned_params
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }

                // 分割参数名和类型
                let parts: Vec<&str> = trimmed.split(':').collect();
                if parts.len() != 2 {
                    return None;
                }

                let param_name = parts[0].trim();
                let param_type = parts[1].trim();

                // 根据类型生成默认值
                let default_value = self.generate_default_value_for_type(param_type);

                Some(format!(
                    "let {}: {} = {};",
                    param_name, param_type, default_value
                ))
            })
            .collect();

        if definitions.is_empty() {
            String::new()
        } else {
            definitions.join("\n        ")
        }
    }

    fn generate_default_value_for_type(&self, param_type: &str) -> String {
        match param_type {
            "&str" => "\"test\"".to_string(),
            "String" => "\"test\".to_string()".to_string(),
            "i32" | "i64" | "u32" | "u64" | "i8" | "i16" | "u8" | "u16" | "usize" | "isize" => {
                "0".to_string()
            }
            "f32" | "f64" => "0.0".to_string(),
            "bool" => "false".to_string(),
            "Vec<String>" => "vec![]".to_string(),
            "Vec<i32>" | "Vec<i64>" | "Vec<u32>" | "Vec<u64>" => "vec![]".to_string(),
            _ => {
                // 对于复杂类型，尝试生成默认值
                if param_type.starts_with("Vec<") {
                    "vec![]".to_string()
                } else if param_type.starts_with("Option<") {
                    "None".to_string()
                } else {
                    // 对于其他类型，尝试使用 Default trait
                    format!("Default::default()")
                }
            }
        }
    }

    // 生成 A 函数 - db_agent.rs 中的函数
    fn generate_db_agent_function(&self, rust_function_name: &str) -> String {
        let return_type = if self.callback_return_type.is_empty() {
            "bool".to_string()
        } else {
            self.callback_return_type.clone()
        };

        let params_with_ref = self.add_ref_to_str_params();
        let param_names_for_call = self.extract_param_names_for_db_worker_call();

        // 生成 &str 参数的转换代码
        let str_conversions = self.generate_str_to_string_conversions_for_db_agent();

        format!(
            r#"pub async fn {}(
    &self,
    {},
) -> Result<{}, EngineError> {{
    // 1. 基础参数转化（需要将数据转为 db 模块的类型）
{}
    // 2. 创建通道和 db_worker
    let (resp_tx, resp_rx) = oneshot::channel();
    let db_worker_clone = self.db_worker.clone();

    // 3. 创建 task，调用 db_worker 对应方法。
    // task 只负责调用简单的方法，复杂逻辑挪到 db 模块内
    let task = Box::pin(async move {{
        let db_worker = db_worker_clone.read().await;
        let result = db_worker.{}({})
            .await;
        let _ = resp_tx.send(result);
    }});

    // 4. 发任务给 db 模块执行
    self.execute(task, resp_rx).await
}}"#,
            rust_function_name,
            params_with_ref,
            return_type,
            str_conversions,
            rust_function_name,
            param_names_for_call
        )
    }

    // 生成 B 函数 - db_worker.rs 中的函数
    fn generate_db_worker_function(&self, rust_function_name: &str) -> String {
        let return_type = if self.callback_return_type.is_empty() {
            "bool".to_string()
        } else {
            self.callback_return_type.clone()
        };

        let params_with_ref = self.add_ref_to_str_params();
        let param_names = self.extract_param_names();

        format!(
            r#"pub async fn {}(
    &self,
    {},
) -> Result<{}, DbError> {{
    log_db_i!("P-{}-T");
    let method_name = "{}";
    let db_lock = self.db_sqlite_lock.read().await;
    let db = db_lock
        .as_ref()
        .ok_or_else(|| self.callback_error(method_name, DbError::NotOpen))?;
    let ret = db.{}({})
        .await
        .unwrap_or_else(|join_error| Err(DbErrorInfo::from_join_error(join_error)));
    self.callback(method_name, ret)
}}"#,
            rust_function_name,
            params_with_ref,
            return_type,
            rust_function_name,
            rust_function_name,
            rust_function_name,
            param_names
        )
    }

    // 生成 C 函数 - db_sqlite.rs 中的函数
    fn generate_db_sqlite_function(&self, rust_function_name: &str) -> String {
        let return_type = if self.callback_return_type.is_empty() {
            "bool".to_string()
        } else {
            self.callback_return_type.clone()
        };

        let params_with_ref = self.add_ref_to_str_params();

        // 生成 &str 参数的转换代码（在函数体内）
        let str_conversions = self.generate_str_conversions_in_function_body();

        format!(
            r#"pub fn {}(
    &self,
    {},
) -> JoinHandle<Result<{}, DbErrorInfo>> {{
    let db_lock_clone = self.db_lock.clone();
{}
    spawn_blocking(move || {{
        let db = db_lock_clone
                .read()
                .map_err(|error| DbErrorInfo::from_lock(error))?;
            let mut transaction_err_opt = None;
            let transaction_ret = db.run_transaction(|_| {{

                if let Err(exp) = ret {{
                    transaction_err_opt = Some(DbErrorInfo::from(exp));
                    return false;
                }}

                return true; //返回 false 回滚整个事务
            }});
            if let Some(error) = transaction_err_opt {{
                return Err(error);
            }}
            if let Err(exp) = transaction_ret {{
                return Err(DbErrorInfo::from(exp));
            }}
            Ok(())
    }})
}}"#,
            rust_function_name, params_with_ref, return_type, str_conversions
        )
    }

    // 辅助函数：生成 db_agent 中 &str 参数的转换代码
    fn generate_str_to_string_conversions_for_db_agent(&self) -> String {
        let cleaned_params = self.clean_params(&self.function_params);
        let conversions: Vec<String> = cleaned_params
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }

                if trimmed.contains(": &str") {
                    let param_name = trimmed.split(':').next()?.trim();
                    Some(format!(
                        "    let {} = {}.to_string();",
                        param_name, param_name
                    ))
                } else {
                    None
                }
            })
            .collect();

        if conversions.is_empty() {
            String::new()
        } else {
            conversions.join("\n") + "\n"
        }
    }

    // 辅助函数：生成调用 db_worker 时的参数列表
    fn extract_param_names_for_db_worker_call(&self) -> String {
        self.clean_params(&self.function_params)
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }

                let param_name = trimmed.split(':').next()?.trim();

                // 如果参数类型是 &str，在调用时需要使用 .as_str()
                if trimmed.contains(": &str") {
                    Some(format!("{}.as_str()", param_name))
                } else {
                    Some(param_name.to_string())
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    // 辅助函数：生成 db_sqlite 中 &str 参数的转换代码（在 spawn_blocking 外部）
    fn generate_str_conversions_in_function_body(&self) -> String {
        let cleaned_params = self.clean_params(&self.function_params);
        let conversions: Vec<String> = cleaned_params
            .split(',')
            .filter_map(|param| {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    return None;
                }

                if trimmed.contains(": &str") {
                    let param_name = trimmed.split(':').next()?.trim();
                    Some(format!(
                        "    let {} = {}.to_string();",
                        param_name, param_name
                    ))
                } else {
                    None
                }
            })
            .collect();

        if conversions.is_empty() {
            String::new()
        } else {
            conversions.join("\n") + "\n"
        }
    }
}

fn java_to_rust_naming(java_name: &str) -> String {
    let mut result = String::new();
    let mut chars = java_name.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            if !result.is_empty() {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }

    result
}

fn pascal_to_snake_case(pascal_name: &str) -> String {
    let mut result = String::new();
    let mut chars = pascal_name.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            if !result.is_empty() {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }

    result
}

fn convert_java_params_to_rust(java_params: &str) -> String {
    java_params
        .split(',')
        .filter_map(|param| {
            let trimmed = param.trim().trim_end_matches(',').trim();
            if trimmed.is_empty() {
                return None;
            }

            // 去掉 final 关键字
            let without_final = trimmed.replace("final ", "");

            // 找到最后一个单词作为变量名
            // 类型部分可能是 String[], List<String>, Map<String, Integer> 等
            let parts: Vec<&str> = without_final.split_whitespace().collect();
            if parts.is_empty() {
                return None;
            }

            // 最后一个是变量名
            let var_name = parts[parts.len() - 1].trim_end_matches(',');

            // 前面的是类型，对于 String[] 这样的类型不应该有空格
            let java_type = if parts.len() == 1 {
                // 只有变量名，没有类型，跳过
                return None;
            } else {
                // 类型可能包含空格（如泛型），但 String[] 不应该有空格
                // 为了简化，我们直接拼接
                parts[0..parts.len() - 1].join("")
            };

            // 转换Java类型到Rust类型
            let rust_type = convert_java_type_to_rust(&java_type);

            // 将Java驼峰命名转换为Rust下划线命名
            let rust_var_name = java_to_rust_naming(var_name);

            Some(format!("{}: {}", rust_var_name, rust_type))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn convert_java_type_to_rust(java_type: &str) -> String {
    let java_type = java_type.trim();

    // 处理数组类型
    if java_type.ends_with("[]") {
        let base_type = java_type.trim_end_matches("[]").trim();
        // 对于数组中的String，使用String而不是&str，因为Vec需要拥有所有权
        let rust_base_type = match base_type {
            "String" => "String".to_string(),
            "int" => "i32".to_string(),
            "long" => "i64".to_string(),
            "short" => "i16".to_string(),
            "byte" => "i8".to_string(),
            "boolean" => "bool".to_string(),
            "float" => "f32".to_string(),
            "double" => "f64".to_string(),
            "char" => "char".to_string(),
            _ => base_type.to_string(),
        };
        return format!("Vec<{}>", rust_base_type);
    }

    // 基本类型映射
    match java_type {
        "String" => "&str".to_string(),
        "int" => "i32".to_string(),
        "long" => "i64".to_string(),
        "short" => "i16".to_string(),
        "byte" => "i8".to_string(),
        "boolean" => "bool".to_string(),
        "float" => "f32".to_string(),
        "double" => "f64".to_string(),
        "char" => "char".to_string(),
        // 自定义类型保持不变
        _ => java_type.to_string(),
    }
}

fn to_pascal_case(snake_case: &str) -> String {
    snake_case
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

#[test]
fn search_messages_by_user_for_channels() {
    SHARED_RUNTIME.block_on(async {
        const ROOM_NAME: &str = "test_room";
        let server_api = ServerApi::new();
        if !server_api.is_chatroom_exist(ROOM_NAME).await {
            server_api.create_chatroom(ROOM_NAME).await;
        }
        TESTER_A.connect().await.unwrap();
        let engine = &TESTER_A.engine;
        let (tx, rx) = oneshot::channel();
        let target_id: &str = "test";
        let channel_ids: Vec<String> = vec![];
        let user_id: &str = "test";
        let start_time: i64 = 0;
        let limit: i32 = 0;
        engine
            .search_messages_by_user_for_channels(
                target_id,
                channel_ids,
                user_id,
                start_time,
                limit,
                |ret| {
                    println!("search_messages_by_user_for_channels: {:?}", ret);
                    assert!(ret.is_ok());
                    tx.send(()).unwrap();
                },
            )
            .await;

        match rx.await {
            Ok(_) => {}
            Err(e) => {
                debug!("search_messages_by_user_for_channels err: {:?}", e);
                assert!(false);
            }
        }
    });
}
