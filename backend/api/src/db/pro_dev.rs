use crate::translate::translate_text;
use anyhow::Context;
use serde_json::value::Value;
use shared::domain::{
    additional_resource::{AdditionalResource, AdditionalResourceId as AddId, ResourceContent},
    asset::{DraftOrLive, PrivacyLevel},
    category::CategoryId,
    meta::ResourceTypeId as TypeId,
    module::{LiteModule, ModuleId, ModuleKind},
    pro_dev::{unit::ProDevUnitId, ProDevData, ProDevId, ProDevResponse},
    user::{UserId, UserScope},
};
use sqlx::{types::Json, PgConnection, PgPool};
use std::collections::HashMap;
use tracing::{instrument, Instrument};
use uuid::Uuid;

use crate::error;

pub(crate) mod additional_resource;
pub(crate) mod module;
pub(crate) mod unit;

pub async fn create(
    pool: &PgPool,
    display_name: &str,
    categories: &[CategoryId],
    creator_id: UserId,
    language: &str,
    description: &str,
) -> Result<ProDevId, CreateProDevError> {
    let mut txn = pool.begin().await?;

    let draft_id = create_pro_dev_data(
        &mut txn,
        display_name,
        categories,
        language,
        description,
        DraftOrLive::Draft,
    )
    .await?;

    let live_id = create_pro_dev_data(
        &mut txn,
        display_name,
        categories,
        language,
        description,
        DraftOrLive::Live,
    )
    .await?;

    let pro_dev = sqlx::query!(
        //language=SQL
        r#"insert into pro_dev (creator_id, author_id, live_id, draft_id) values ($1, $1, $2, $3) returning id"#,
        creator_id.0,
        live_id,
        draft_id,
    )
    .fetch_one(&mut txn)
    .await?;

    txn.commit().await?;

    Ok(ProDevId(pro_dev.id))
}

pub async fn create_pro_dev_data(
    txn: &mut PgConnection,
    display_name: &str,
    categories: &[CategoryId],
    language: &str,
    description: &str,
    draft_or_live: DraftOrLive,
) -> Result<Uuid, CreateProDevError> {
    log::warn!("description: {}", description);

    let pro_dev_data = sqlx::query!(
        // language=SQL
        r#"
insert into pro_dev_data
   (display_name, language, description, draft_or_live)
values ($1, $2, $3, $4)
returning id
"#,
        display_name,
        language,
        description,
        draft_or_live as i16,
    )
    .fetch_one(&mut *txn)
    .await?;

    super::recycle_metadata(&mut *txn, "pro_dev_data", pro_dev_data.id, categories).await?;

    Ok(pro_dev_data.id)
}

pub enum CreateProDevError {
    Sqlx(sqlx::Error),
    InternalServerError(anyhow::Error),
}

impl From<sqlx::Error> for CreateProDevError {
    fn from(e: sqlx::Error) -> Self {
        Self::Sqlx(e)
    }
}

impl From<anyhow::Error> for CreateProDevError {
    fn from(e: anyhow::Error) -> Self {
        Self::InternalServerError(e)
    }
}

pub async fn get_one(
    pool: &PgPool,
    id: ProDevId,
    draft_or_live: DraftOrLive,
) -> anyhow::Result<Option<ProDevResponse>> {
    let res = sqlx::query!( //language=SQL
        r#"
with cte as (
    select id      as "pro_dev_id",
           creator_id,
           author_id,
           likes,
           plays,
           live_up_to_date,
           case
               when $2 = 0 then pro_dev.draft_id
               when $2 = 1 then pro_dev.live_id
               end as "draft_or_live_id",
           published_at
    from pro_dev
    where id = $1
)
select cte.pro_dev_id                                          as "pro_dev_id: ProDevId",
       display_name,
       creator_id                                             as "creator_id?: UserId",
       author_id                                              as "author_id?: UserId",
       (select given_name || ' '::text || family_name
        from user_profile
        where user_profile.user_id = author_id)            as "author_name",
       published_at,
       updated_at,
       privacy_level                                       as "privacy_level!: PrivacyLevel",
       language,
       description,
       translated_description                              as "translated_description!: Json<HashMap<String, String>>",
       likes,
       plays,
       live_up_to_date,
       other_keywords,
       translated_keywords,
       duration,
       (
            select row(pro_dev_data_module.id, kind, is_complete)
            from pro_dev_data_module
            where pro_dev_data_id = cte.draft_or_live_id and "index" = 0
            order by "index"
        )                                                   as "cover?: (ModuleId, ModuleKind, bool)",
       array(select row (category_id)
             from pro_dev_data_category
             where pro_dev_data_id = cte.draft_or_live_id)     as "categories!: Vec<(CategoryId,)>",
       array(
             select row (pddr.id, pddr.display_name, resource_type_id, resource_content)
             from pro_dev_data_resource "pddr"
             where pddr.pro_dev_data_id = cte.draft_or_live_id
       )                                                    as "additional_resource!: Vec<(AddId, String, TypeId, Value)>",
       array(
            select row(pddu.unit_id)
            from pro_dev_data_unit "pddu"
            where pddu.pro_dev_data_id = pro_dev_data.id
            order by "index"
    )                                                     as "units!: Vec<(ProDevUnitId,)>"
from pro_dev_data
         inner join cte on cte.draft_or_live_id = pro_dev_data.id
"#,
        id.0,
        draft_or_live as i16,
    )
        .fetch_optional(pool).await?;

    let pro_dev = res.map(|row| ProDevResponse {
        id: row.pro_dev_id,
        published_at: row.published_at,
        creator_id: row.creator_id,
        author_id: row.author_id,
        author_name: row.author_name,
        likes: row.likes,
        plays: row.plays,
        live_up_to_date: row.live_up_to_date,
        pro_dev_data: ProDevData {
            draft_or_live,
            display_name: row.display_name,
            language: row.language,
            categories: row.categories.into_iter().map(|(it,)| it).collect(),
            last_edited: row.updated_at,
            description: row.description,
            duration: row.duration.map(|x| x as u32),
            cover: row.cover.map(|(id, kind, is_complete)| LiteModule {
                id,
                kind,
                is_complete,
            }),
            additional_resources: row
                .additional_resource
                .into_iter()
                .map(
                    |(id, display_name, resource_type_id, resource_content)| AdditionalResource {
                        id,
                        display_name,
                        resource_type_id,
                        resource_content: serde_json::from_value::<ResourceContent>(
                            resource_content,
                        )
                        .unwrap(),
                    },
                )
                .collect(),
            privacy_level: row.privacy_level,
            other_keywords: row.other_keywords,
            translated_keywords: row.translated_keywords,
            translated_description: row.translated_description.0,
            units: row.units.into_iter().map(|(it,)| it).collect(),
        },
    });

    Ok(pro_dev)
}

pub async fn get_by_ids(
    db: &PgPool,
    ids: &[Uuid],
    draft_or_live: DraftOrLive,
) -> sqlx::Result<Vec<ProDevResponse>> {
    let mut txn = db.begin().await?;

    let pro_dev = sqlx::query!(
        //language=SQL
        r#"
select pro_dev.id                               as "id!: ProDevId",
       creator_id                               as "creator_id?: UserId",
       author_id                                as "author_id?: UserId",
       (select given_name || ' '::text || family_name
        from user_profile
        where user_profile.user_id = author_id) as "author_name",
       live_id                                  as "live_id!",
       draft_id                                 as "draft_id!",
       published_at,
       likes                                    as "likes!",
       plays                                    as "plays!",
       live_up_to_date                          as "live_up_to_date!"
from pro_dev
inner join unnest($1::uuid[])
    with ordinality t(id, ord) using (id)
order by ord asc
    "#,
        ids,
    )
    .fetch_all(&mut txn)
    .await?;

    let pro_dev_data_ids: Vec<Uuid> = match draft_or_live {
        DraftOrLive::Draft => pro_dev.iter().map(|it| it.draft_id).collect(),
        DraftOrLive::Live => pro_dev.iter().map(|it| it.live_id).collect(),
    };

    let pro_dev_data = sqlx::query!(
        //language=SQL
        r#"
select  id,
        display_name                                       as "display_name!",
        updated_at,
        privacy_level                                      as "privacy_level!: PrivacyLevel",
        language                                           as "language!",
        description                                         as "description!",
        translated_description                              as "translated_description!: Json<HashMap<String, String>>",
        duration                                           as "duration",
        other_keywords                             as "other_keywords!",
        translated_keywords                        as "translated_keywords!",
        (
            select row(pro_dev_data_module.id, kind, is_complete)
            from pro_dev_data_module
            where pro_dev_data_id = pro_dev_data.id and "index" = 0
            order by "index"
        )                                                   as "cover?: (ModuleId, ModuleKind, bool)",
        array(select row (category_id)
            from pro_dev_data_category
            where pro_dev_data_id = pro_dev_data.id)     as "categories!: Vec<(CategoryId,)>",
        array(
            select row (pddr.id, pddr.display_name, resource_type_id, resource_content)
            from pro_dev_data_resource "pddr"
            where pddr.pro_dev_data_id = pro_dev_data.id
        )                                                    as "additional_resource!: Vec<(AddId, String, TypeId, Value)>",
        array(
            select row(unit_id)
            from pro_dev_data_unit "pddu"
            where pddu.pro_dev_data_id = pro_dev_data.id
            order by "index"
        )                                                     as "units!: Vec<(ProDevUnitId,)>"
from pro_dev_data
inner join unnest($1::uuid[])
    with ordinality t(id, ord) using (id)
order by ord asc
"#,
        &pro_dev_data_ids
    )
        .fetch_all(&mut txn)
        .await?;

    let v = pro_dev
        .into_iter()
        .zip(pro_dev_data.into_iter())
        .map(|(pro_dev_row, pro_dev_data_row)| ProDevResponse {
            id: pro_dev_row.id,
            published_at: pro_dev_row.published_at,
            creator_id: pro_dev_row.creator_id,
            author_id: pro_dev_row.author_id,
            author_name: pro_dev_row.author_name,
            likes: pro_dev_row.likes,
            plays: pro_dev_row.plays,
            live_up_to_date: pro_dev_row.live_up_to_date,
            pro_dev_data: ProDevData {
                draft_or_live,
                display_name: pro_dev_data_row.display_name,
                language: pro_dev_data_row.language,
                duration: pro_dev_data_row.duration.map(|x| x as u32),
                cover: pro_dev_data_row
                    .cover
                    .map(|(id, kind, is_complete)| LiteModule {
                        id,
                        kind,
                        is_complete,
                    }),
                categories: pro_dev_data_row
                    .categories
                    .into_iter()
                    .map(|(it,)| it)
                    .collect(),
                last_edited: pro_dev_data_row.updated_at,
                description: pro_dev_data_row.description,
                additional_resources: pro_dev_data_row
                    .additional_resource
                    .into_iter()
                    .map(|(id, display_name, resource_type_id, resource_content)| {
                        AdditionalResource {
                            id,
                            display_name,
                            resource_type_id,
                            resource_content: serde_json::from_value::<ResourceContent>(
                                resource_content,
                            )
                            .unwrap(),
                        }
                    })
                    .collect(),
                privacy_level: pro_dev_data_row.privacy_level,
                other_keywords: pro_dev_data_row.other_keywords,
                translated_keywords: pro_dev_data_row.translated_keywords,
                translated_description: pro_dev_data_row.translated_description.0,
                units: pro_dev_data_row.units.into_iter().map(|(it,)| it).collect(),
            },
        })
        .collect();

    txn.rollback().await?;

    Ok(v)
}

#[instrument(skip(db))]
pub async fn browse(
    db: &sqlx::Pool<sqlx::Postgres>,
    author_id: Option<UserId>,
    draft_or_live: Option<DraftOrLive>,
    privacy_level: Vec<PrivacyLevel>,
    page: i32,
    page_limit: u32,
    resource_types: Vec<Uuid>,
) -> sqlx::Result<Vec<ProDevResponse>> {
    let mut txn = db.begin().await?;

    let privacy_level: Vec<i16> = privacy_level.iter().map(|x| *x as i16).collect();

    let pro_dev_data = sqlx::query!(
    //language=SQL
    r#"
with cte as (
    select (array_agg(pdd.id))[1]
    from pro_dev_data "pdd"
          left join pro_dev on (draft_id = pdd.id or (live_id = pdd.id and pdd.last_synced_at is not null and published_at is not null))
          left join pro_dev_data_resource "resource" on pdd.id = resource.pro_dev_data_id
    where (author_id = $1 or $1 is null)
        and (pdd.draft_or_live = $2 or $2 is null)
        and (pdd.privacy_level = any($3) or $3 = array[]::smallint[])
        and (resource.resource_type_id = any($4) or $4 = array[]::uuid[])
    group by coalesce(updated_at, created_at)
    order by coalesce(updated_at, created_at) desc
),
cte1 as (
    select * from unnest(array(select cte.array_agg from cte)) with ordinality t(id
   , ord) order by ord
)
select pro_dev.id                                                                as "pro_dev_id: ProDevId",
    privacy_level                                                               as "privacy_level: PrivacyLevel",
    creator_id                                                                  as "creator_id?: UserId",
    author_id                                                                   as "author_id?: UserId",
    (select given_name || ' '::text || family_name
     from user_profile
     where user_profile.user_id = author_id)                                     as "author_name",
    published_at,
    likes,
    plays,
    duration,
    live_up_to_date,
    display_name                                                                  as "display_name!",
    updated_at,
    language                                                                      as "language!",
    description                                                                   as "description!",
    translated_description                                                        as "translated_description!: Json<HashMap<String,String>>",
    draft_or_live                                                                 as "draft_or_live!: DraftOrLive",
    other_keywords                                                                as "other_keywords!",
    translated_keywords                                                           as "translated_keywords!",
    (
        select row(pro_dev_data_module.id, kind, is_complete)
        from pro_dev_data_module
        where pro_dev_data_id = pro_dev_data.id and "index" = 0
        order by "index"
    )                                                   as "cover?: (ModuleId, ModuleKind, bool)",
    array(select row (category_id)
            from pro_dev_data_category
            where pro_dev_data_id = pro_dev_data.id)     as "categories!: Vec<(CategoryId,)>",
    array(select row (id, display_name, resource_type_id, resource_content)
                from pro_dev_data_resource
                where pro_dev_data_id = pro_dev_data.id
          )                                          as "additional_resource!: Vec<(AddId, String, TypeId, Value)>",
    array(
            select row(unit_id)
            from pro_dev_data_unit "pddu"
            where pddu.pro_dev_data_id = pro_dev_data.id
            order by "index"
    )                                                     as "units!: Vec<(ProDevUnitId,)>"
from cte1
inner join pro_dev_data on cte1.id = pro_dev_data.id
inner join pro_dev on (
    pro_dev_data.id = pro_dev.draft_id
    or (
        pro_dev_data.id = pro_dev.live_id
        and last_synced_at is not null
        and pro_dev.published_at is not null
    )
)
where ord > (1 * $5 * $6)
order by ord asc
limit $6
"#,
    author_id.map(|it| it.0),
    draft_or_live.map(|it| it as i16),
    &privacy_level[..],
    &resource_types[..],
    page,
    page_limit as i32,
)
    .fetch_all(&mut txn)
    .instrument(tracing::info_span!("query pro_dev_data"))
    .await?;

    let v: Vec<_> = pro_dev_data
        .into_iter()
        .map(|pro_dev_data_row| ProDevResponse {
            id: pro_dev_data_row.pro_dev_id,
            published_at: pro_dev_data_row.published_at,
            creator_id: pro_dev_data_row.creator_id,
            author_id: pro_dev_data_row.author_id,
            author_name: pro_dev_data_row.author_name,
            likes: pro_dev_data_row.likes,
            plays: pro_dev_data_row.plays,
            live_up_to_date: pro_dev_data_row.live_up_to_date,
            pro_dev_data: ProDevData {
                draft_or_live: pro_dev_data_row.draft_or_live,
                display_name: pro_dev_data_row.display_name,
                language: pro_dev_data_row.language,
                cover: pro_dev_data_row
                    .cover
                    .map(|(id, kind, is_complete)| LiteModule {
                        id,
                        kind,
                        is_complete,
                    }),
                categories: pro_dev_data_row
                    .categories
                    .into_iter()
                    .map(|(it,)| it)
                    .collect(),
                last_edited: pro_dev_data_row.updated_at,
                description: pro_dev_data_row.description,
                duration: pro_dev_data_row.duration.map(|x| x as u32),
                additional_resources: pro_dev_data_row
                    .additional_resource
                    .into_iter()
                    .map(|(id, display_name, resource_type_id, resource_content)| {
                        AdditionalResource {
                            id,
                            display_name,
                            resource_type_id,
                            resource_content: serde_json::from_value::<ResourceContent>(
                                resource_content,
                            )
                            .unwrap(),
                        }
                    })
                    .collect(),
                privacy_level: pro_dev_data_row.privacy_level,
                other_keywords: pro_dev_data_row.other_keywords,
                translated_keywords: pro_dev_data_row.translated_keywords,
                translated_description: pro_dev_data_row.translated_description.0,
                units: pro_dev_data_row.units.into_iter().map(|(it,)| it).collect(),
            },
        })
        .collect();

    txn.rollback().await?;

    Ok(v)
}

pub async fn update_draft(
    pool: &PgPool,
    api_key: &Option<String>,
    id: ProDevId,
    display_name: Option<&str>,
    categories: Option<&[CategoryId]>,
    language: Option<&str>,
    description: Option<&str>,
    privacy_level: Option<PrivacyLevel>,
    other_keywords: Option<String>,
) -> Result<(), error::UpdateWithMetadata> {
    let mut txn = pool.begin().await?;

    let draft_id = sqlx::query!(
        //language=SQL
        r#"
select draft_id from pro_dev join pro_dev_data on pro_dev.draft_id = pro_dev_data.id where pro_dev.id = $1 for update
"#,
        id.0
    )
    .fetch_optional(&mut txn)
    .await?
    .ok_or(error::UpdateWithMetadata::ResourceNotFound)?
    .draft_id;

    if let Some(privacy_level) = privacy_level {
        sqlx::query!(
            //language=SQL
            r#"
update pro_dev_data
set privacy_level = coalesce($2, privacy_level),
    updated_at = now()
where id = $1
  and $2 is distinct from privacy_level
    "#,
            draft_id,
            privacy_level as i16,
        )
        .execute(&mut txn)
        .await?;
    }

    if let Some(description) = description {
        sqlx::query!(
            r#"
update pro_dev_data
set description = $2,
    updated_at = now()
where id = $1 and $2 is distinct from description"#,
            draft_id,
            description,
        )
        .execute(&mut txn)
        .await?;
    }

    if let Some(other_keywords) = other_keywords {
        let translate_text = match &api_key {
            Some(key) => translate_text(&other_keywords, "he", "en", key)
                .await
                .context("could not translate text")?,
            None => None,
        };

        sqlx::query!(
            r#"
update pro_dev_data
set other_keywords = $2,
    translated_keywords = (case when ($3::text is not null) then $3::text else (translated_keywords) end),
    updated_at = now()
where id = $1 and $2 is distinct from other_keywords"#,
            draft_id,
            other_keywords,
            translate_text
        )
        .execute(&mut *txn)
        .await?;
    }

    if let Some(display_name) = display_name {
        sqlx::query!(
            r#"
update pro_dev_data
set display_name = $2,
    updated_at = now()
where id = $1 and $2 is distinct from display_name"#,
            draft_id,
            display_name,
        )
        .execute(&mut txn)
        .await?;
    }

    // update trivial, not null fields
    sqlx::query!(
        //language=SQL
        r#"
update pro_dev_data
set language         = coalesce($2, language),
    updated_at = now()
where id = $1
  and ($2::text is not null and $2 is distinct from language)
"#,
        draft_id,
        language,
    )
    .execute(&mut txn)
    .await?;

    if let Some(categories) = categories {
        super::recycle_metadata(&mut txn, "pro_dev_data", draft_id, categories)
            .await
            .map_err(super::meta::handle_metadata_err)?;
    }

    txn.commit().await?;

    Ok(())
}

pub async fn delete(pool: &PgPool, id: ProDevId) -> Result<(), error::Delete> {
    let mut txn = pool.begin().await?;

    let (draft_id, live_id) = get_draft_and_live_ids(&mut txn, id)
        .await
        .ok_or(error::Delete::ResourceNotFound)?;

    sqlx::query!(
        //language=SQL
        r#"
with del_data as (
    delete from pro_dev_data
        where id is not distinct from $1 or id is not distinct from $2)
delete
from pro_dev
where id is not distinct from $3

"#,
        draft_id,
        live_id,
        id.0,
    )
    .execute(&mut txn)
    .await?;

    txn.commit().await?;

    Ok(())
}

// `None` here means do not filter.
#[instrument(skip(db))]
pub async fn filtered_count(
    db: &PgPool,
    privacy_level: Vec<PrivacyLevel>,
    author_id: Option<UserId>,
    draft_or_live: Option<DraftOrLive>,
    resource_types: Vec<Uuid>,
) -> sqlx::Result<(u64, u64)> {
    let privacy_level: Vec<i16> = privacy_level.iter().map(|x| *x as i16).collect();

    let pro_dev_data = sqlx::query!(
        //language=SQL
        r#"
        with cte as (
            select (array_agg(pdd.id))[1]
            from pro_dev_data "pdd"
                  inner join pro_dev on (draft_id = pdd.id or (live_id = pdd.id and pdd.last_synced_at is not null and published_at is not null))
                  left join pro_dev_data_resource "resource" on pdd.id = resource.pro_dev_data_id
            where (author_id = $1 or $1 is null)
                and (pdd.draft_or_live = $2 or $2 is null)
                and (pdd.privacy_level = any($3) or $3 = array[]::smallint[])
                and (resource.resource_type_id = any($4) or $4 = array[]::uuid[])
            group by coalesce(updated_at, created_at)
        )
        select count(*) as "count!" from unnest(array(select cte.array_agg from cte)) with ordinality t(id, ord)
    "#,
        author_id.map(|it| it.0),
        draft_or_live.map(|it| it as i16),
        &privacy_level[..],
        &resource_types[..]
    )
    .fetch_one(db)
    .await?;

    let pro_dev = sqlx::query!(
        //language=SQL
        r#"
        with cte as (
            select (array_agg(pro_dev.id))[1]
            from pro_dev_data "cd"
                  inner join pro_dev on (draft_id = cd.id or (live_id = cd.id and cd.last_synced_at is not null and published_at is not null))
                  left join pro_dev_data_resource "resource" on cd.id = resource.pro_dev_data_id
            where (author_id = $1 or $1 is null)
                and (cd.draft_or_live = $2 or $2 is null)
                and (cd.privacy_level = any($3) or $3 = array[]::smallint[])
                and (resource.resource_type_id = any($4) or $4 = array[]::uuid[])
            group by coalesce(updated_at, created_at)
        )
        select count(*) as "count!" from unnest(array(select cte.array_agg from cte)) with ordinality t(id, ord)
"#,
        author_id.map(|it| it.0),
        draft_or_live.map(|it| it as i16),
        &privacy_level[..],
        &resource_types[..]
    )
    .fetch_one(db)
    .await?;

    Ok((pro_dev.count as u64, pro_dev_data.count as u64))
}

pub async fn get_draft_and_live_ids(
    txn: &mut PgConnection,
    pro_dev_id: ProDevId,
) -> Option<(Uuid, Uuid)> {
    sqlx::query!(
        //language=SQL
        r#"
select draft_id, live_id from pro_dev where id = $1
"#,
        pro_dev_id.0,
    )
    .fetch_optional(&mut *txn)
    .await
    .ok()?
    .map(|it| (it.draft_id, it.live_id))
}

/// Clones a copy of the pro_dev data and modules, preserving the module's stable IDs
pub async fn clone_data(
    txn: &mut PgConnection,
    from_data_id: &Uuid,
    draft_or_live: DraftOrLive,
) -> Result<Uuid, error::CloneDraft> {
    println!("here in clone");
    let new_id = sqlx::query!(
        //language=SQL
        r#"
insert into pro_dev_data
(display_name, created_at, updated_at, language, last_synced_at, description, privacy_level, other_keywords, translated_keywords, translated_description)
select display_name,
       created_at,
       updated_at,
       language,
       last_synced_at,
       description,
       privacy_level,
       other_keywords,
       translated_keywords,
       translated_description::jsonb
from pro_dev_data
where id = $1
returning id
        "#,
        from_data_id,
    )
    .fetch_one(&mut *txn)
    .await?
    .id;

    update_draft_or_live(txn, new_id, draft_or_live).await?;

    sqlx::query!(
        //language=SQL
        r#"
 insert into pro_dev_data_module ("index", pro_dev_data_id, kind, is_complete, contents)
 select "index", $2 as "pro_dev_id", kind, is_complete, contents
 from pro_dev_data_module
 where pro_dev_data_id = $1
            "#,
        from_data_id,
        new_id,
    )
    .execute(&mut *txn)
    .await?;

    sqlx::query!(
        //language=SQL
        r#"
insert into pro_dev_data_resource(pro_dev_data_id, resource_type_id, display_name, resource_content)
select $2, resource_type_id, display_name, resource_content
from pro_dev_data_resource
where pro_dev_data_id = $1
        "#,
        from_data_id,
        new_id,
    )
    .execute(&mut *txn)
    .await?;

    sqlx::query!(
        //language=SQL
        r#"
insert into pro_dev_data_category(pro_dev_data_id, category_id)
select $2, category_id
from pro_dev_data_category
where pro_dev_data_id = $1
        "#,
        from_data_id,
        new_id,
    )
    .execute(&mut *txn)
    .await?;

    sqlx::query!(
        //language=SQL
        r#"
    insert into pro_dev_data_unit(pro_dev_data_id, display_name, description, index, value)
    select $2, display_name, description, index, value 
    from pro_dev_data_unit
    where pro_dev_data_id = $1
            "#,
        from_data_id,
        new_id,
    )
    .execute(&mut *txn)
    .await?;

    Ok(new_id)
}

pub async fn is_admin(db: &PgPool, user_id: UserId) -> Result<bool, error::Auth> {
    let authed = sqlx::query!(
        r#"
select exists(select 1 from user_scope where user_id = $1 and scope = any($2)) as "authed!"
"#,
        user_id.0,
        &[UserScope::Admin as i16, UserScope::AdminJig as i16][..],
    )
    .fetch_one(db)
    .await?
    .authed;

    if !authed {
        return Ok(false);
    }

    Ok(true)
}

pub async fn authz(
    db: &PgPool,
    user_id: UserId,
    pro_dev_id: Option<ProDevId>,
) -> Result<(), error::Auth> {
    let authed = match pro_dev_id {
        None => {
            sqlx::query!(
                r#"
select exists(select 1 from user_scope where user_id = $1 and scope = any($2)) as "authed!"
"#,
                user_id.0,
                &[
                    UserScope::Admin as i16,
                    UserScope::AdminJig as i16,
                    UserScope::ManageSelfJig as i16,
                ][..],
            )
            .fetch_one(db)
            .await?
            .authed
        }
        Some(id) => {
            sqlx::query!(
                //language=SQL
                r#"
select exists (
    select 1 from user_scope where user_id = $1 and scope = any($2)
) or (
    exists (select 1 from user_scope where user_id = $1 and scope = $3) and
    not exists (select 1 from pro_dev where pro_dev.id = $4 and pro_dev.author_id <> $1)
) as "authed!"
"#,
                user_id.0,
                &[UserScope::Admin as i16, UserScope::AdminJig as i16,][..],
                UserScope::ManageSelfJig as i16,
                id.0
            )
            .fetch_one(db)
            .await?
            .authed
        }
    };

    if !authed {
        return Err(error::Auth::Forbidden);
    }

    Ok(())
}

async fn update_draft_or_live(
    conn: &mut PgConnection,
    pro_dev_data_id: Uuid,
    draft_or_live: DraftOrLive,
) -> sqlx::Result<()> {
    sqlx::query!(
        //language=SQL
        r#"
update pro_dev_data
set draft_or_live = $2
where id = $1
            "#,
        pro_dev_data_id,
        draft_or_live as i16
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}
