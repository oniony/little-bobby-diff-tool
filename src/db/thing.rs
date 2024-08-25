#[derive(Clone, PartialEq)]
pub struct Column {
    pub table_name: String,
    pub column_name: String,
    pub ordinal_position: i32,
    pub column_default: Option<String>,
    pub is_nullable: String,
    pub data_type: String,
    pub character_maximum_length: Option<i32>,
    pub numeric_precision: Option<i32>,
    pub numeric_scale: Option<i32>,
    pub datetime_precision: Option<i32>,
    pub is_identity: String,
    pub identity_generation: Option<String>,
    pub is_generated: String,
    pub generation_expression: Option<String>,
    pub is_updatable: String,
}

#[derive(Clone, PartialEq)]
pub struct Privilege {
    pub grantor: String,
    pub grantee: String,
    pub privilege_type: String,
    pub is_grantable: String,
}

#[derive(Clone, PartialEq)]
pub struct Routine {
    pub signature: String,
    pub routine_type: Option<String>,
    pub data_type: Option<String>,
    pub type_udt_name: Option<String>,
    pub routine_body: String,
    pub routine_definition: Option<String>,
    pub external_name: Option<String>,
    pub external_language: String,
    pub is_deterministic: String,
    pub is_null_call: Option<String>,
    pub security_type: String,
}

#[derive(Clone, PartialEq)]
pub struct Schema {
    pub schema_name: String,
    pub schema_owner: String,
}

#[derive(Clone, PartialEq)]
pub struct Sequence {
    pub sequence_name: String,
    pub data_type: String,
    pub numeric_precision: i32,
    pub numeric_precision_radix: i32,
    pub numeric_scale: i32,
    pub start_value: String,
    pub minimum_value: String,
    pub maximum_value: String,
    pub increment: String,
    pub cycle_option: String,
}

#[derive(Clone, PartialEq)]
pub struct Table {
    pub table_name: String,
    pub table_type: String,
    pub is_insertable_into: String,
}

#[derive(Clone, PartialEq)]
pub struct TableConstraint {
    pub constraint_name: String,
    pub constraint_type: String,
    pub is_deferrable: String,
    pub initially_deferred: String,
    pub nulls_distinct: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct Trigger {
    pub trigger_name: String,
    pub event_manipulation: String,
    pub action_order: i32,
    pub action_condition: Option<String>,
    pub action_statement: String,
    pub action_orientation: String,
    pub action_timing: String,
    pub action_reference_old_table: Option<String>,
    pub action_reference_new_table: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct View {
    pub view_name: String,
    pub view_definition: Option<String>,
    pub check_option: String,
    pub is_updatable: String,
    pub is_insertable_into: String,
    pub is_trigger_updatable: String,
    pub is_trigger_deletable: String,
    pub is_trigger_insertable_into: String,
}
