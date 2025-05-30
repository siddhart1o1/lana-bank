use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Display, str::FromStr};

pub use audit::AuditInfo;
pub use authz::{action_description::*, AllOrOne};

#[cfg(feature = "governance")]
es_entity::entity_id! {
    UserId;
    UserId => governance::CommitteeMemberId,
}
#[cfg(not(feature = "governance"))]
es_entity::entity_id! { UserId }

es_entity::entity_id! { AuthenticationId, PermissionSetId, RoleId }

pub const PERMISSION_SET_USER_WRITER: &str = "user_writer";
pub const PERMISSION_SET_USER_READER: &str = "user_reader";

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct RoleName(Cow<'static, str>);
impl RoleName {
    /// Name of the role that will have all permission sets.
    pub const SUPERUSER: RoleName = RoleName(Cow::Borrowed("superuser"));

    // Transitional roles before they are replaced by seeded roles
    pub const ACCOUNTANT: RoleName = RoleName(Cow::Borrowed("accountant"));
    pub const BANK_MANAGER: RoleName = RoleName(Cow::Borrowed("bank-manager"));
    pub const ADMIN: RoleName = RoleName(Cow::Borrowed("admin"));

    pub fn new(role_name: impl Into<String>) -> Self {
        RoleName(Cow::Owned(role_name.into()))
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

impl Display for RoleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name().fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreAccessAction {
    User(UserAction),
    Role(RoleAction),
}

impl CoreAccessAction {
    pub const ROLE_CREATE: Self = CoreAccessAction::Role(RoleAction::Create);
    pub const ROLE_UPDATE: Self = CoreAccessAction::Role(RoleAction::Update);

    pub const USER_CREATE: Self = CoreAccessAction::User(UserAction::Create);
    pub const USER_READ: Self = CoreAccessAction::User(UserAction::Read);
    pub const USER_LIST: Self = CoreAccessAction::User(UserAction::List);
    pub const USER_ASSIGN_ROLE: Self = CoreAccessAction::User(UserAction::AssignRole);
    pub const USER_REVOKE_ROLE: Self = CoreAccessAction::User(UserAction::RevokeRole);
    pub const USER_UPDATE_AUTHENTICATION_ID: Self =
        CoreAccessAction::User(UserAction::UpdateAuthenticationId);

    pub fn entities() -> Vec<(
        CoreAccessActionDiscriminants,
        Vec<ActionDescription<NoPath>>,
    )> {
        use CoreAccessActionDiscriminants::*;

        let mut result = vec![];

        for entity in <CoreAccessActionDiscriminants as strum::VariantArray>::VARIANTS {
            let actions = match entity {
                User => UserAction::describe(),
                Role => RoleAction::describe(),
            };

            result.push((*entity, actions));
        }

        result
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum RoleAction {
    Create,
    Update,
}

impl RoleAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => ActionDescription::new(variant, &[PERMISSION_SET_USER_WRITER]),
                Self::Update => ActionDescription::new(variant, &[PERMISSION_SET_USER_WRITER]),
            };
            res.push(action_description);
        }

        res
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum UserAction {
    Read,
    Create,
    List,
    Update,
    AssignRole,
    RevokeRole,
    UpdateAuthenticationId,
}

impl UserAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => ActionDescription::new(variant, &[PERMISSION_SET_USER_WRITER]),
                Self::Read => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_USER_READER, PERMISSION_SET_USER_WRITER],
                ),
                Self::List => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_USER_READER, PERMISSION_SET_USER_WRITER],
                ),
                Self::Update => ActionDescription::new(variant, &[PERMISSION_SET_USER_WRITER]),
                Self::AssignRole => ActionDescription::new(variant, &[PERMISSION_SET_USER_WRITER]),
                Self::RevokeRole => ActionDescription::new(variant, &[PERMISSION_SET_USER_WRITER]),
                Self::UpdateAuthenticationId => {
                    ActionDescription::new(variant, &[PERMISSION_SET_USER_WRITER])
                }
            };
            res.push(action_description);
        }

        res
    }
}

impl Display for CoreAccessAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreAccessActionDiscriminants::from(self))?;
        use CoreAccessAction::*;
        match self {
            User(action) => action.fmt(f),
            Role(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreAccessAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use CoreAccessActionDiscriminants::*;
        let res = match entity.parse()? {
            User => CoreAccessAction::from(action.parse::<UserAction>()?),
            Role => CoreAccessAction::from(action.parse::<RoleAction>()?),
        };
        Ok(res)
    }
}

impl From<UserAction> for CoreAccessAction {
    fn from(action: UserAction) -> Self {
        CoreAccessAction::User(action)
    }
}

impl From<RoleAction> for CoreAccessAction {
    fn from(action: RoleAction) -> Self {
        CoreAccessAction::Role(action)
    }
}

pub type UserAllOrOne = AllOrOne<UserId>;
pub type RoleAllOrOne = AllOrOne<RoleId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants, strum::EnumCount)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreAccessObject {
    User(UserAllOrOne),
    Role(RoleAllOrOne),
}

impl CoreAccessObject {
    pub const fn all_roles() -> CoreAccessObject {
        CoreAccessObject::Role(AllOrOne::All)
    }
    pub const fn role(id: RoleId) -> CoreAccessObject {
        CoreAccessObject::Role(AllOrOne::ById(id))
    }

    pub const fn all_users() -> CoreAccessObject {
        CoreAccessObject::User(AllOrOne::All)
    }
    pub fn user(id: impl Into<Option<UserId>>) -> CoreAccessObject {
        match id.into() {
            Some(id) => CoreAccessObject::User(AllOrOne::ById(id)),
            None => CoreAccessObject::all_users(),
        }
    }
}

impl Display for CoreAccessObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreAccessObjectDiscriminants::from(self);
        use CoreAccessObject::*;
        match self {
            User(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            Role(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
        }
    }
}

impl FromStr for CoreAccessObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreAccessObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            User => {
                let obj_ref = id.parse().map_err(|_| "could not parse UserObject")?;
                CoreAccessObject::User(obj_ref)
            }
            Role => {
                let obj_ref = id.parse().map_err(|_| "could not parse RoleObject")?;
                CoreAccessObject::Role(obj_ref)
            }
        };
        Ok(res)
    }
}
