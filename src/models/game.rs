
pub fn xp_for_next_level(level: i64) -> i64 {
    100 * level.max(1)
}

pub fn level_from_xp(total_xp: i64) -> (i64, i64) {
    let mut level = 1i64;
    let mut remaining = total_xp.max(0);
    loop {
        let need = xp_for_next_level(level);
        if remaining >= need {
            remaining -= need;
            level += 1;
        } else {
            return (level, need - remaining);
        }
    }
}

pub mod villages {
    use sea_orm::entity::prelude::*;
    use sea_orm::ActiveValue::Set;
    use serde::{Deserialize, Serialize};
    use validator::Validate;
    
    
    use uuid::Uuid;
    

    fn now() -> String {
        chrono::Utc::now().to_rfc3339()
    }

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "villages")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: String,
        pub user_id: i64,
        pub name: String,
        pub level: i64,
        pub xp: i64,
        pub xp_to_next: i64,
        pub gold: f64,
        pub gems: f64,
        pub elixir: f64,
        pub trophy: i64,
        pub shield_until: Option<String>,
        pub last_tick_at: Option<String>,
        pub created_at: Option<String>,
        pub updated_at: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
    pub struct CreateVillageRequest {
        pub name: Option<String>,
    }

    impl CreateVillageRequest {
        pub fn into_active(self, user_id: i64) -> ActiveModel {
            ActiveModel {
                id: Set(uuid::Uuid::new_v4().to_string()),
                user_id: Set(user_id),
                name: Set(self.name.unwrap_or_else(|| "My Fortress".to_string())),
                level: Set(1),
                xp: Set(0),
                xp_to_next: Set(100),
                gold: Set(500.0),
                gems: Set(20.0),
                elixir: Set(0.0),
                trophy: Set(0),
                shield_until: Set(None),
                last_tick_at: Set(Some(now())),
                created_at: Set(Some(now())),
                updated_at: Set(Some(now())),
                ..Default::default()
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VillageResponse {
        pub id: String,
        pub user_id: i64,
        pub name: String,
        pub level: i64,
        pub xp: i64,
        pub xp_to_next: i64,
        pub gold: f64,
        pub gems: f64,
        pub elixir: f64,
        pub trophy: i64,
        pub shield_until: Option<String>,
        pub last_tick_at: Option<String>,
        pub created_at: Option<String>,
        pub updated_at: Option<String>,
    }

    impl From<Model> for VillageResponse {
        fn from(m: Model) -> Self {
            Self {
                id: m.id,
                user_id: m.user_id,
                name: m.name,
                level: m.level,
                xp: m.xp,
                xp_to_next: m.xp_to_next,
                gold: m.gold,
                gems: m.gems,
                elixir: m.elixir,
                trophy: m.trophy,
                shield_until: m.shield_until,
                last_tick_at: m.last_tick_at,
                created_at: m.created_at,
                updated_at: m.updated_at,
            }
        }
    }
}

pub mod buildings {
    use sea_orm::entity::prelude::*;
    use sea_orm::ActiveValue::Set;
    use sea_orm::IntoActiveModel;
    use serde::{Deserialize, Serialize};
    use validator::Validate;
    

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "buildings")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: String,
        pub village_id: String,
        pub user_id: i64,
        pub building_type: String,
        pub source_kind: String,
        pub source_id: Option<String>,
        pub name: String,
        pub level: i64,
        pub grid_x: i64,
        pub grid_y: i64,
        pub hp: i64,
        pub max_hp: i64,
        pub production_rate: f64,
        pub stored_resource: f64,
        pub last_collected_at: Option<String>,
        pub upgrade_cost: f64,
        pub is_upgrading: bool,
        pub upgrade_finishes_at: Option<String>,
        pub created_at: Option<String>,
        pub updated_at: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    impl Model {
        pub fn pending_production(&self, now: &str) -> f64 {
            let (start, now_ts) = match (
                self.last_collected_at.as_deref(),
                chrono::DateTime::parse_from_rfc3339(now),
            ) {
                (Some(s), Ok(n)) => match chrono::DateTime::parse_from_rfc3339(s) {
                    Ok(s) => (s, n),
                    Err(_) => return 0.0,
                },
                _ => return 0.0,
            };
            let hours = (now_ts - start).num_seconds() as f64 / 3600.0;
            if hours <= 0.0 {
                0.0
            } else {
                self.production_rate * hours
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct UpdateBuildingRequest {
        pub name: Option<String>,
        pub grid_x: Option<i64>,
        pub grid_y: Option<i64>,
        pub production_rate: Option<f64>,
    }

    impl UpdateBuildingRequest {
        pub fn apply(self, current: &Model) -> ActiveModel {
            let mut am = current.clone().into_active_model();
            if let Some(v) = self.name {
                am.name = Set(v);
            }
            if let Some(v) = self.grid_x {
                am.grid_x = Set(v);
            }
            if let Some(v) = self.grid_y {
                am.grid_y = Set(v);
            }
            if let Some(v) = self.production_rate {
                am.production_rate = Set(v);
            }
            am.updated_at = Set(Some(
                chrono::Utc::now().to_rfc3339(),
            ));
            am
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BuildingResponse {
        pub id: String,
        pub village_id: String,
        pub user_id: i64,
        pub building_type: String,
        pub source_kind: String,
        pub source_id: Option<String>,
        pub name: String,
        pub level: i64,
        pub grid_x: i64,
        pub grid_y: i64,
        pub hp: i64,
        pub max_hp: i64,
        pub production_rate: f64,
        pub stored_resource: f64,
        pub last_collected_at: Option<String>,
        pub upgrade_cost: f64,
        pub is_upgrading: bool,
        pub upgrade_finishes_at: Option<String>,
        pub created_at: Option<String>,
        pub updated_at: Option<String>,
        pub pending_production: f64,
    }

    impl BuildingResponse {
        pub fn from_model(m: Model, now: &str) -> Self {
            let pending_production = m.pending_production(now);
            Self {
                id: m.id,
                village_id: m.village_id,
                user_id: m.user_id,
                building_type: m.building_type,
                source_kind: m.source_kind,
                source_id: m.source_id,
                name: m.name,
                level: m.level,
                grid_x: m.grid_x,
                grid_y: m.grid_y,
                hp: m.hp,
                max_hp: m.max_hp,
                production_rate: m.production_rate,
                stored_resource: m.stored_resource,
                last_collected_at: m.last_collected_at,
                upgrade_cost: m.upgrade_cost,
                is_upgrading: m.is_upgrading,
                upgrade_finishes_at: m.upgrade_finishes_at,
                created_at: m.created_at,
                updated_at: m.updated_at,
                pending_production,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CollectResponse {
        pub building_id: String,
        pub collected: f64,
        pub stored_resource: f64,
        pub gold: f64,
        pub gems: f64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UpgradeResponse {
        pub building_id: String,
        pub level: i64,
        pub is_upgrading: bool,
        pub upgrade_finishes_at: Option<String>,
        pub gold: f64,
        pub gems: f64,
    }
}

pub mod troops {
    use sea_orm::entity::prelude::*;
    use sea_orm::ActiveValue::Set;
    use serde::{Deserialize, Serialize};
    use validator::Validate;
    
    
    

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "troops")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: String,
        pub user_id: i64,
        pub troop_type: String,
        pub name: String,
        pub level: i64,
        pub attack: i64,
        pub hp: i64,
        pub count: i64,
        pub train_cost: f64,
        pub train_time_secs: i64,
        pub training_finishes_at: Option<String>,
        pub expense_id: Option<String>,
        pub created_at: Option<String>,
        pub updated_at: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
    pub struct TrainTroopRequest {
        #[validate(range(min = 1, max = 50, message = "count must be 1..=50"))]
        pub count: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TroopResponse {
        pub id: String,
        pub user_id: i64,
        pub troop_type: String,
        pub name: String,
        pub level: i64,
        pub attack: i64,
        pub hp: i64,
        pub count: i64,
        pub train_cost: f64,
        pub train_time_secs: i64,
        pub training_finishes_at: Option<String>,
        pub expense_id: Option<String>,
        pub created_at: Option<String>,
        pub updated_at: Option<String>,
    }

    impl From<Model> for TroopResponse {
        fn from(m: Model) -> Self {
            Self {
                id: m.id,
                user_id: m.user_id,
                troop_type: m.troop_type,
                name: m.name,
                level: m.level,
                attack: m.attack,
                hp: m.hp,
                count: m.count,
                train_cost: m.train_cost,
                train_time_secs: m.train_time_secs,
                training_finishes_at: m.training_finishes_at,
                expense_id: m.expense_id,
                created_at: m.created_at,
                updated_at: m.updated_at,
            }
        }
    }
}

pub mod achievements {
    use sea_orm::entity::prelude::*;
    use sea_orm::ActiveValue::Set;
    use serde::{Deserialize, Serialize};
    
    

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "achievements")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: String,
        #[sea_orm(unique)]
        pub code: String,
        pub title: String,
        pub description: String,
        pub icon: String,
        pub xp_reward: i64,
        pub gem_reward: f64,
        pub requirement_kind: String,
        pub requirement_value: i64,
        pub tier: String,
        pub sort_order: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AchievementResponse {
        pub id: String,
        pub code: String,
        pub title: String,
        pub description: String,
        pub icon: String,
        pub xp_reward: i64,
        pub gem_reward: f64,
        pub requirement_kind: String,
        pub requirement_value: i64,
        pub tier: String,
        pub sort_order: i64,
        pub unlocked: bool,
        pub progress: i64,
        pub is_claimed: bool,
    }
}

pub mod user_achievements {
    use sea_orm::entity::prelude::*;
    use sea_orm::ActiveValue::Set;
    use serde::{Deserialize, Serialize};
    
    

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "user_achievements")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: String,
        pub user_id: i64,
        pub achievement_id: String,
        pub unlocked_at: Option<String>,
        pub progress: i64,
        pub is_claimed: bool,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UserAchievementResponse {
        pub id: String,
        pub user_id: i64,
        pub achievement_id: String,
        pub unlocked_at: Option<String>,
        pub progress: i64,
        pub is_claimed: bool,
    }

    impl From<Model> for UserAchievementResponse {
        fn from(m: Model) -> Self {
            Self {
                id: m.id,
                user_id: m.user_id,
                achievement_id: m.achievement_id,
                unlocked_at: m.unlocked_at,
                progress: m.progress,
                is_claimed: m.is_claimed,
            }
        }
    }
}

pub mod battles {
    use sea_orm::entity::prelude::*;
    use sea_orm::ActiveValue::Set;
    use serde::{Deserialize, Serialize};
    use validator::Validate;
    
    
    

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "battles")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: String,
        pub user_id: i64,
        pub battle_type: String,
        pub status: String,
        pub stars: i64,
        pub destruction_percent: f64,
        pub gold_reward: f64,
        pub gem_reward: f64,
        pub xp_reward: i64,
        pub trophy_change: i64,
        pub enemy_troops: Option<String>,
        pub enemy_buildings: Option<String>,
        pub created_at: Option<String>,
        pub updated_at: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
    pub struct StartBattleRequest {
        pub battle_type: String,
        pub troop_ids: Vec<String>,
        pub target_summary: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BattleResponse {
        pub id: String,
        pub user_id: i64,
        pub battle_type: String,
        pub status: String,
        pub stars: i64,
        pub destruction_percent: f64,
        pub gold_reward: f64,
        pub gem_reward: f64,
        pub xp_reward: i64,
        pub trophy_change: i64,
        pub enemy_troops: Option<String>,
        pub enemy_buildings: Option<String>,
        pub created_at: Option<String>,
        pub updated_at: Option<String>,
    }

    impl From<Model> for BattleResponse {
        fn from(m: Model) -> Self {
            Self {
                id: m.id,
                user_id: m.user_id,
                battle_type: m.battle_type,
                status: m.status,
                stars: m.stars,
                destruction_percent: m.destruction_percent,
                gold_reward: m.gold_reward,
                gem_reward: m.gem_reward,
                xp_reward: m.xp_reward,
                trophy_change: m.trophy_change,
                enemy_troops: m.enemy_troops,
                enemy_buildings: m.enemy_buildings,
                created_at: m.created_at,
                updated_at: m.updated_at,
            }
        }
    }
}

pub use achievements::{
    AchievementResponse, ActiveModel as AchievementActiveModel, Model as Achievement,
};
pub use battles::{
    BattleResponse, StartBattleRequest, ActiveModel as BattleActiveModel, Model as Battle,
};
pub use buildings::{
    BuildingResponse, CollectResponse, UpdateBuildingRequest, UpgradeResponse,
    ActiveModel as BuildingActiveModel, Model as Building,
};
pub use troops::{
    TrainTroopRequest, TroopResponse, ActiveModel as TroopActiveModel, Model as Troop,
};
pub use user_achievements::{
    ActiveModel as UserAchievementActiveModel, Model as UserAchievement, UserAchievementResponse,
};
pub use villages::{
    CreateVillageRequest, VillageResponse, ActiveModel as VillageActiveModel, Model as Village,
};
