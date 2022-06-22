//! Simplified FGD representation.
//! Some parts of specification missing.
//! Reference: https://developer.valvesoftware.com/wiki/FGD

use glam::{UVec3, Vec3};
use serde::{Deserialize, Serialize};

const TAB: &str = "    ";

mod to_fgd_literal;
pub use to_fgd_literal::*;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct FgdFile {
    pub name: String,
    pub includes: Vec<String>,
    pub classes: Vec<FgdClass>,
}

impl FgdFile {
    pub fn serialize(&self) -> String {
        let mut output = String::new();

        if !self.includes.is_empty() {
            for include in &self.includes {
                output.push_str("@include ");
                output.push_str(include);
                output.push('\n');
            }

            output.push('\n');
        }

        let mut iter = self.classes.iter().peekable();

        while let Some(class) = iter.next() {
            output.push_str(&class.serialize());
            output.push('\n');

            if iter.peek().is_some() {
                output.push('\n');
            }
        }

        output
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct FgdClass {
    pub class_type: FgdClassType,
    pub name: String,
    pub description: String,
    pub class_properties: Vec<FgdClassProperty>,
    pub entity_properties: Vec<EntityProperty>,
}

impl FgdClass {
    fn serialize(&self) -> String {
        // Class type
        let mut output = format!("{} ", self.class_type.serialize());

        // Class properties
        for class_prop in &self.class_properties {
            output.push_str(&class_prop.serialize());
            output.push(' ');
        }

        output.push_str("= ");
        output.push_str(&self.name);
        output.push_str(" :\n");

        // Description (split by line)
        let mut iter = self.description.lines().peekable();

        while let Some(line) = iter.next() {
            output.push_str(TAB);

            if iter.peek().is_some() {
                output.push_str(&(line.to_owned() + "\\n").to_fgd_literal());
                output.push_str(" +\n");
            } else {
                output.push_str(&line.to_fgd_literal());
                output.push('\n');
            }
        }

        // Entity properties
        output.push_str("[\n");

        for entity_prop in &self.entity_properties {
            for line in entity_prop.serialize().lines() {
                output.push_str(TAB);
                output.push_str(line);
                output.push('\n');
            }
        }

        output.push(']');

        output
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum FgdClassType {
    Base,
    Point,
    Solid,
}

impl FgdClassType {
    fn serialize(&self) -> &str {
        match self {
            FgdClassType::Base => "@BaseClass",
            FgdClassType::Point => "@PointClass",
            FgdClassType::Solid => "@SolidClass",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum FgdClassProperty {
    Base(Vec<String>),
    Model(String),
    Color(UVec3),
    Size(Vec3, Vec3),
}

impl FgdClassProperty {
    pub fn serialize(&self) -> String {
        match self {
            Self::Base(base_classes) => format!("base({})", base_classes.join(", ")),
            Self::Model(model_path) => format!("model({})", model_path.to_fgd_literal()),
            Self::Color(color) => format!("color({} {} {})", color.x, color.y, color.z),
            Self::Size(p1, p2) => format!(
                "size({} {} {}, {} {} {})",
                p1.x, p1.y, p1.z, p2.x, p2.y, p2.z
            ),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum EntityProperty {
    String(EntityPropertyData<String>),
    Integer(EntityPropertyData<i32>),
    Boolean(EntityPropertyData<bool>),
    Float(EntityPropertyData<f32>),
    Choices(EntityPropertyData<i32>, Vec<Choice>),
    Flags(FlagsData),
}

impl EntityProperty {
    pub fn name(&self) -> &String {
        match self {
            EntityProperty::String(data) => &data.name,
            EntityProperty::Integer(data) => &data.name,
            EntityProperty::Boolean(data) => &data.name,
            EntityProperty::Float(data) => &data.name,
            EntityProperty::Choices(data, _) => &data.name,
            EntityProperty::Flags(data) => &data.name,
        }
    }

    pub fn set_name(&mut self, new_name: String) {
        match self {
            EntityProperty::String(data) => data.name = new_name,
            EntityProperty::Integer(data) => data.name = new_name,
            EntityProperty::Boolean(data) => data.name = new_name,
            EntityProperty::Float(data) => data.name = new_name,
            EntityProperty::Choices(data, _) => data.name = new_name,
            EntityProperty::Flags(data) => data.name = new_name,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            EntityProperty::String(_) => "String",
            EntityProperty::Integer(_) => "Integer",
            EntityProperty::Boolean(_) => "Boolean",
            EntityProperty::Float(_) => "Float",
            EntityProperty::Choices(_, _) => "Choices",
            EntityProperty::Flags(_) => "Flags",
        }
    }

    fn serialize(&self) -> String {
        match self {
            EntityProperty::String(data) => data.serialize("string"),
            EntityProperty::Integer(data) => data.serialize("integer"),
            EntityProperty::Boolean(data) => data.serialize("boolean"),
            EntityProperty::Float(data) => data.serialize("float"),
            EntityProperty::Choices(data, choices) => {
                let mut output = data.serialize("choices");
                output.push_str(" =\n[\n");

                for choice in choices {
                    output.push_str(TAB);
                    output.push_str(&choice.serialize());
                    output.push('\n');
                }

                output.push(']');

                output
            }
            EntityProperty::Flags(data) => data.serialize(),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct EntityPropertyData<T: ToFgdLiteral + Default> {
    pub name: String,
    pub display_name: String,
    pub default: T,
    pub description: String,
}

impl<T: ToFgdLiteral + Default> EntityPropertyData<T> {
    pub fn named(name: String) -> Self {
        EntityPropertyData::<T> {
            name,
            ..Default::default()
        }
    }
}

impl<T: ToFgdLiteral + Default> EntityPropertyData<T> {
    fn serialize(&self, type_name: &str) -> String {
        format!(
            "{}({}) : {} : {} : {}",
            self.name,
            type_name,
            self.display_name.to_fgd_literal(),
            self.default.to_fgd_literal(),
            self.description.to_fgd_literal()
        )
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Choice {
    pub index: i32,
    pub name: String,
}

impl Choice {
    fn serialize(&self) -> String {
        format!("{} : {}", self.index, self.name.to_fgd_literal())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct FlagsData {
    pub name: String,
    pub flags: Vec<Flag>,
}

impl FlagsData {
    fn serialize(&self) -> String {
        let mut output = format!("{}(flags) =\n[\n", self.name);

        for flag in &self.flags {
            output.push_str(TAB);
            output.push_str(&flag.serialize());
            output.push('\n');
        }

        output.push(']');

        output
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Flag {
    pub flag: i32,
    pub name: String,
    pub default: bool,
}

impl Flag {
    fn serialize(&self) -> String {
        format!(
            "{} : {} : {}",
            self.flag,
            self.name.to_fgd_literal(),
            self.default.to_fgd_literal(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Choice, EntityProperty, EntityPropertyData, FgdClass, FgdClassProperty, FgdClassType,
        FgdFile, Flag, FlagsData,
    };
    use glam::UVec3;

    #[test]
    fn test_fgd_serialize() {
        let fgd = FgdFile {
            name: "entities".to_string(),
            includes: vec!["base1".to_string(), "base2".to_string()],
            classes: vec![
                FgdClass {
                    class_type: FgdClassType::Base,
                    name: "TestBase".to_string(),
                    description: "Test base class\nMultiline description".to_string(),
                    class_properties: vec![FgdClassProperty::Color(UVec3::new(255, 255, 255))],
                    entity_properties: vec![EntityProperty::Float(EntityPropertyData::<f32> {
                        name: "test_prop".to_string(),
                        display_name: "Test property".to_string(),
                        default: 5.0,
                        description: "This is a test".to_string(),
                    })],
                },
                FgdClass {
                    class_type: FgdClassType::Point,
                    name: "test_point".to_string(),
                    description: "Test point class".to_string(),
                    class_properties: vec![FgdClassProperty::Base(vec!["TestBase".to_string()])],
                    entity_properties: vec![EntityProperty::Flags(FlagsData {
                        name: "spawnflags".to_string(),
                        flags: vec![
                            Flag {
                                flag: 1,
                                name: "Flag 1".to_string(),
                                default: true,
                            },
                            Flag {
                                flag: 2,
                                name: "Flag 2".to_string(),
                                default: false,
                            },
                            Flag {
                                flag: 4,
                                name: "Flag 3".to_string(),
                                default: true,
                            },
                        ],
                    })],
                },
            ],
        };

        assert_eq!(include_str!("test_data/test_output.fgd"), fgd.serialize());
    }

    #[test]
    fn test_basic_property_serialize() {
        let property = EntityProperty::String(EntityPropertyData::<String> {
            name: "test".to_string(),
            display_name: "Test".to_string(),
            default: "yes".to_string(),
            description: "This is a test".to_string(),
        });

        assert_eq!(
            property.serialize(),
            "test(string) : \"Test\" : \"yes\" : \"This is a test\""
        );
    }

    #[test]
    fn test_choices_property_serialize() {
        let property = EntityProperty::Choices(
            EntityPropertyData::<i32> {
                name: "test2".to_string(),
                display_name: "Test 2".to_string(),
                default: 0,
                description: "This is another test".to_string(),
            },
            vec![
                Choice {
                    index: 0,
                    name: "Choice 1".to_string(),
                },
                Choice {
                    index: 1,
                    name: "Choice 2".to_string(),
                },
            ],
        );

        assert_eq!(
            property.serialize(),
            r#"test2(choices) : "Test 2" : 0 : "This is another test" =
[
    0 : "Choice 1"
    1 : "Choice 2"
]"#
        );
    }

    #[test]
    fn test_flags_data_serialize() {
        let property = EntityProperty::Flags(FlagsData {
            name: "spawnflags".to_string(),
            flags: vec![
                Flag {
                    flag: 1,
                    name: "This is a flag".to_string(),
                    default: false,
                },
                Flag {
                    flag: 2,
                    name: "This is another flag".to_string(),
                    default: true,
                },
            ],
        });

        assert_eq!(
            property.serialize(),
            r#"spawnflags(flags) =
[
    1 : "This is a flag" : 0
    2 : "This is another flag" : 1
]"#
        );
    }
}
