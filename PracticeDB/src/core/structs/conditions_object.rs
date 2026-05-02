use crate::core::LogicalConnector;
use crate::core::Operand;

#[derive(Default, Debug)]
pub struct ConditionsObject {
    pub object_one: String,
    pub object_two: String,
    pub object_one_is_field: bool,
    pub object_two_is_field: bool,
    pub operand: Operand,
    pub connector: Option<LogicalConnector>,
}
