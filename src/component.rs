use anyhow::{Context, Error, Result};
use std::collections::HashMap;

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum InputId {
    Filter,
    Key,
    Value,
}

#[derive(Clone, Debug)]
pub enum MainInput {
    None,
    Filter,
}

impl TryFrom<MainInput> for InputId {
    type Error = Error;
    fn try_from(input: MainInput) -> Result<Self, Self::Error> {
        match input {
            MainInput::Filter => Ok(InputId::Filter),
            _ => Err(Error::msg("Cannot convert into an input.")),
        }
    }
}

impl TryFrom<&MainInput> for &InputId {
    type Error = Error;
    fn try_from(input: &MainInput) -> Result<Self, Self::Error> {
        match *input {
            MainInput::Filter => Ok(&InputId::Filter),
            _ => Err(Error::msg("Cannot convert into an input.")),
        }
    }
}

#[derive(Clone, Debug)]
pub enum EditingInput {
    Key,
    Value,
}

impl From<EditingInput> for InputId {
    fn from(input: EditingInput) -> Self {
        match input {
            EditingInput::Key => InputId::Key,
            EditingInput::Value => InputId::Value,
        }
    }
}

impl From<&EditingInput> for &InputId {
    fn from(input: &EditingInput) -> Self {
        match *input {
            EditingInput::Key => &InputId::Key,
            EditingInput::Value => &InputId::Value,
        }
    }
}

pub struct InputField {
    content: String,
    is_active: bool,
}

impl InputField {
    pub fn new() -> Result<Self> {
        Ok(InputField {
            content: String::new(),
            is_active: false,
        })
    }
}

pub struct InputArena {
    fields: HashMap<InputId, InputField>,
}

impl InputArena {
    pub fn new() -> Result<Self> {
        let mut fields = HashMap::with_capacity(3); // hardcoded for now
        fields.insert(InputId::Filter, InputField::new()?);
        fields.insert(InputId::Key, InputField::new()?);
        fields.insert(InputId::Value, InputField::new()?);
        Ok(InputArena { fields })
    }

    pub fn get(&self, k: &InputId) -> Result<&InputField> {
        self.fields
            .get(k)
            .context(format!("Cannot find {k:?} in the input arena"))
    }

    pub fn get_content(&self, k: &InputId) -> Result<&String> {
        Ok(&self
            .fields
            .get(k)
            .context(format!("Cannot find {k:?} in the input arena"))?
            .content)
    }

    pub fn get_mut(&mut self, k: &InputId) -> Result<&mut InputField> {
        self.fields
            .get_mut(k)
            .context(format!("Cannot find {k:?} in the input arena"))
    }

    pub fn value_pop(&mut self, k: &InputId) -> Result<()> {
        self.get_mut(k)?.content.pop();
        Ok(())
    }

    pub fn value_push(&mut self, k: &InputId, value: char) -> Result<()> {
        self.get_mut(k)?.content.push(value);
        Ok(())
    }
}
