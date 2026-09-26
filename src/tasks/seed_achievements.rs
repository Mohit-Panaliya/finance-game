use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::game_engine::achievements::{self as engine_achievements, DEFS};
use crate::models::game::achievements::{Entity as AchievementEntity, Column as AchievementColumn, ActiveModel as AchievementActiveModel};

pub struct SeedAchievements;

#[async_trait]
impl Task for SeedAchievements {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "achievements:seed".to_string(),
            detail: "Seed achievement definitions into the achievements table".to_string(),
        }
    }

    async fn run(&self, app_context: &AppContext, _vars: &task::Vars) -> Result<()> {
        let db = app_context
            .shared_store
            .get_ref::<sea_orm_turso::TursoConnection>()
            .ok_or_else(|| Error::string("database unavailable"))?;

        for def in DEFS {
            let existing = AchievementEntity::find()
                .filter(AchievementColumn::Id.eq(def.id.to_string()))
                .one(&*db)
                .await
                .map_err(|e| Error::string(&e.to_string()))?;

            if existing.is_some() {
                let mut am = AchievementActiveModel {
                    id: sea_orm::ActiveValue::Unchanged(def.id.to_string()),
                    ..Default::default()
                };
                am.code = Set(def.code.to_string());
                am.title = Set(def.title.to_string());
                am.description = Set(def.description.to_string());
                am.icon = Set(def.icon.to_string());
                am.xp_reward = Set(def.xp_reward);
                am.gem_reward = Set(def.gem_reward);
                am.requirement_kind = Set(def.requirement_kind.to_string());
                am.requirement_value = Set(def.requirement_value);
                am.tier = Set(def.tier.to_string());
                am.sort_order = Set(def.sort_order);
                am.update(&*db)
                    .await
                    .map_err(|e| Error::string(&e.to_string()))?;
                tracing::info!(id = def.id, "achievement updated");
            } else {
                AchievementActiveModel {
                    id: Set(def.id.to_string()),
                    code: Set(def.code.to_string()),
                    title: Set(def.title.to_string()),
                    description: Set(def.description.to_string()),
                    icon: Set(def.icon.to_string()),
                    xp_reward: Set(def.xp_reward),
                    gem_reward: Set(def.gem_reward),
                    requirement_kind: Set(def.requirement_kind.to_string()),
                    requirement_value: Set(def.requirement_value),
                    tier: Set(def.tier.to_string()),
                    sort_order: Set(def.sort_order),
                }
                .insert(&*db)
                .await
                .map_err(|e| Error::string(&e.to_string()))?;
                tracing::info!(id = def.id, "achievement seeded");
            }
        }

        println!("Seeded {} achievements", DEFS.len());
        Ok(())
    }
}
