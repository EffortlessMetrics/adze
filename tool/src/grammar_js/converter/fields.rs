use super::GrammarJsConverter;
use adze_ir::FieldId;

impl GrammarJsConverter {
    pub(super) fn get_or_create_field(&mut self, name: &str) -> FieldId {
        // Check if field already exists
        for (field_id, field_name) in &self.fields {
            if field_name == name {
                return *field_id;
            }
        }

        // Create new field
        let field_id = FieldId(self.next_field_id.try_into().unwrap());
        self.fields.insert(field_id, name.to_string());
        self.next_field_id += 1;
        field_id
    }
}
