# parameters

## Responsibility

Test CLI parameter validation across all command parameters.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `mod.rs` | Export parameter test modules |
| `agent_id_parameter_test.rs` | Test Agent Id parameter validation |
| `agent_ids_parameter_test.rs` | Test Agent Ids parameter validation |
| `api_key_parameter_test.rs` | Test Api Key parameter validation |
| `description_parameter_test.rs` | Test Description parameter validation |
| `email_parameter_test.rs` | Test Email parameter validation |
| `endpoint_parameter_test.rs` | Test Endpoint parameter validation |
| `export_format_parameter_test.rs` | Test Export Format parameter validation |
| `format_parameter_test.rs` | Test Format parameter validation |
| `id_parameter_test.rs` | Test Id parameter validation |
| `message_parameter_test.rs` | Test Message parameter validation |
| `name_parameter_test.rs` | Test Name parameter validation |
| `new_password_parameter_test.rs` | Test New Password parameter validation |
| `output_file_parameter_test.rs` | Test Output File parameter validation |
| `output_parameter_test.rs` | Test Output parameter validation |
| `password_parameter_test.rs` | Test Password parameter validation |
| `period_parameter_test.rs` | Test Period parameter validation |
| `project_id_parameter_test.rs` | Test Project Id parameter validation |
| `project_parameter_test.rs` | Test Project parameter validation |
| `provider_id_parameter_test.rs` | Test Provider Id parameter validation |
| `provider_ids_parameter_test.rs` | Test Provider Ids parameter validation |
| `provider_parameter_test.rs` | Test Provider parameter validation |
| `role_parameter_test.rs` | Test Role parameter validation |
| `status_parameter_test.rs` | Test Status parameter validation |
| `token_id_parameter_test.rs` | Test Token Id parameter validation |
| `username_parameter_test.rs` | Test Username parameter validation |

## Notes

Each test file validates a specific CLI parameter type including edge cases, validation rules, and error messages. Tests use the handler-validation pattern where handlers validate parameter existence and format before adapters use them.
