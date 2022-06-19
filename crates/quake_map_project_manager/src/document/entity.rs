use super::{DocumentIoContext, DocumentIoError, EditorDocumentItem};
use bevy::{
    prelude::*,
    reflect::TypeRegistry,
    scene::{
        serde::{SceneDeserializer, SceneSerializer},
        serialize_ron,
    },
};
use bevy_quake_map::fgd::FgdClass;
use serde::{
    de::{DeserializeSeed, Error, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};

pub const ENTITIES_DIR: &str = "entities/";

const STRUCT_NAME: &str = "EntityDefinition";
const FIELD_CLASS: &str = "class";
const FIELD_SCENE: &str = "scene";

pub struct EntityDefinition {
    pub class: FgdClass,
    pub scene: Option<DynamicScene>,
}

impl EditorDocumentItem for EntityDefinition {
    fn deserialize(
        serialized: &str,
        doc_context: &DocumentIoContext,
    ) -> Result<Self, DocumentIoError> {
        let mut deserializer = ron::de::Deserializer::from_bytes(serialized.as_bytes())?;
        let def_deserializer = EntityDefinitionDeserializer {
            type_registry: &doc_context.type_registry,
        };

        Ok(def_deserializer.deserialize(&mut deserializer)?)
    }

    fn serialize(&self, doc_context: &DocumentIoContext) -> Result<String, DocumentIoError> {
        let serializer = EntityDefinitionSerializer {
            entity_definition: self,
            type_registry: &doc_context.type_registry,
        };

        Ok(serialize_ron(serializer)?)
    }

    fn save_path(&self) -> String {
        format!("{}/{}.entity.ron", ENTITIES_DIR, self.class.name)
    }

    fn name(&self) -> &str {
        &self.class.name
    }

    fn set_name(&mut self, new_name: &str) {
        self.class.name = new_name.to_string();
    }
}

struct EntityDefinitionSerializer<'a> {
    entity_definition: &'a EntityDefinition,
    type_registry: &'a TypeRegistry,
}

impl<'a> Serialize for EntityDefinitionSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct(STRUCT_NAME, 2)?;

        s.serialize_field(FIELD_CLASS, &self.entity_definition.class)?;
        s.serialize_field(
            FIELD_SCENE,
            &self
                .entity_definition
                .scene
                .as_ref()
                .map(|scene| SceneSerializer {
                    scene,
                    registry: self.type_registry,
                }),
        )?;

        s.end()
    }
}

struct EntityDefinitionDeserializer<'a> {
    type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for EntityDefinitionDeserializer<'a> {
    type Value = EntityDefinition;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            STRUCT_NAME,
            &[FIELD_CLASS, FIELD_SCENE],
            EntityDefinitionVisitor {
                type_registry: self.type_registry,
            },
        )
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum EntityDefinitionField {
    Class,
    Scene,
}

struct EntityDefinitionVisitor<'a> {
    type_registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for EntityDefinitionVisitor<'a> {
    type Value = EntityDefinition;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an entity definition")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut class = None;
        let mut scene = None;

        while let Some(key) = map.next_key()? {
            match key {
                EntityDefinitionField::Class => {
                    if class.is_some() {
                        return Err(Error::duplicate_field(FIELD_CLASS));
                    }

                    class = Some(map.next_value::<FgdClass>()?);
                }
                EntityDefinitionField::Scene => {
                    if scene.is_some() {
                        return Err(Error::duplicate_field(FIELD_SCENE));
                    }

                    scene = Some(map.next_value_seed(OptionalSceneDeserializer {
                        type_registry: self.type_registry,
                    })?);
                }
            }
        }

        let class = class.ok_or_else(|| Error::missing_field(FIELD_CLASS))?;
        let scene = scene.ok_or_else(|| Error::missing_field(FIELD_CLASS))?;

        Ok(EntityDefinition { class, scene })
    }
}

struct OptionalSceneDeserializer<'a> {
    type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for OptionalSceneDeserializer<'a> {
    type Value = Option<DynamicScene>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_option(OptionalSceneVisitor {
            type_registry: self.type_registry,
        })
    }
}

struct OptionalSceneVisitor<'a> {
    type_registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for OptionalSceneVisitor<'a> {
    type Value = Option<DynamicScene>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a scene definition, or None")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let seed = SceneDeserializer {
            type_registry: &self.type_registry.internal.read(),
        };

        Ok(Some(seed.deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use super::EntityDefinition;
    use crate::document::{DocumentIoContext, EditorDocumentItem};
    use bevy::{ecs::entity::EntityMap, prelude::*};
    use bevy_quake_map::fgd::{FgdClass, FgdClassType};

    #[derive(Default, Reflect, Component)]
    #[reflect(Component)]
    struct TestComponent;

    fn scene_setup(mut commands: Commands) {
        commands.spawn().insert(TestComponent);
    }

    #[test]
    fn test_serde() {
        let mut app = App::new();
        app.register_type::<TestComponent>().add_system(scene_setup);
        app.update();

        let ctx = DocumentIoContext::from_world(&mut app.world);

        let def = EntityDefinition {
            class: FgdClass {
                class_type: FgdClassType::Base,
                name: "TestClass".to_string(),
                description: "A test entity class".to_string(),
                class_properties: vec![],
                entity_properties: vec![],
            },
            scene: Some(DynamicScene::from_world(&app.world, &ctx.type_registry)),
        };

        let serialized = def
            .serialize(&ctx)
            .unwrap_or_else(|err| panic!("Serialization failed: {}", err));

        let deserialized = EntityDefinition::deserialize(&serialized, &ctx)
            .unwrap_or_else(|err| panic!("Deserialization failed: {}", err));

        assert_eq!(def.class, deserialized.class);

        let mut world = World::new();
        world.insert_resource(ctx.type_registry);

        deserialized
            .scene
            .unwrap()
            .write_to_world(&mut world, &mut EntityMap::default())
            .unwrap_or_else(|err| panic!("Failed to write scene: {}", err));

        assert_eq!(world.entities().len(), 1);

        let mut query = world.query::<&TestComponent>();
        assert_eq!(query.iter(&world).len(), 1);
    }
}
