use std::rc::Rc;

use components::image::upload::upload_image;
use dominator::clone;
use futures::future::join;
use gloo_timers::future::TimeoutFuture;
use shared::{
    api::endpoints::{self, meta, user, ApiEndpoint},
    domain::{
        image::{user::UserImageCreateRequest, ImageId, ImageSize},
        meta::MetadataResponse,
        user::ResetPasswordRequest,
    },
    error::EmptyError,
    media::MediaLibrary,
};
use wasm_bindgen_futures::spawn_local;

use super::state::{ProfilePage, ResetPasswordStatus};
use utils::{fetch::api_with_auth, prelude::*, unwrap::UnwrapJiExt};
use web_sys::File;

impl ProfilePage {
    pub fn send_reset_password(self: &Rc<Self>) {
        let state = self;

        state
            .reset_password_status
            .set(ResetPasswordStatus::Loading);

        spawn_local(clone!(state => async move {
            let req = ResetPasswordRequest {
                email: state.user.email.get_cloned()
            };

            let res = endpoints::user::ResetPassword::api_no_auth_empty(Some(req)).await;

            match res {
                Ok(_) => {
                    state.reset_password_status.set(ResetPasswordStatus::Sent);
                },
                Err(_err) => {
                    todo!()
                }
            }
            TimeoutFuture::new(5000).await;
            state.reset_password_status.set(ResetPasswordStatus::default());
        }));
    }

    pub fn load_initial_data(self: &Rc<Self>) {
        let state = self;
        state.loader.load(clone!(state => async move {
            join(
                state.load_profile(),
                state.load_metadata()
            ).await;
        }));
    }

    async fn load_profile(self: &Rc<Self>) {
        //let resp:Result<UserProfile, EmptyError> = api_with_auth::< _, _, ()>(&user::Profile::PATH, user::Profile::METHOD, None).await;
        let resp = user::Profile::api_with_auth(None).await;

        self.user.fill_from_user(resp.unwrap_ji());
    }

    async fn load_metadata(self: &Rc<Self>) {
        match api_with_auth::<MetadataResponse, EmptyError, ()>(
            meta::Get::PATH,
            meta::Get::METHOD,
            None,
        )
        .await
        {
            Err(_) => {}
            Ok(res) => {
                self.metadata.set(Some(res));
            }
        };
    }

    pub fn save_profile(self: &Rc<Self>) {
        let state = self;
        state.loader.load(clone!(state => async move {
            let info = state.user.to_update();

            let res = user::PatchProfile::api_with_auth_empty(Some(info)).await;
            if let Err(_err) = res {
                todo!()
            }
        }));
    }

    pub fn set_profile_image(self: &Rc<Self>, file: File) {
        let state = self;
        state.loader.load(clone!(state => async move {
            match upload_profile_image(file).await {
                Err(err) => {
                    log::error!("{}", err);
                },
                Ok(image_id) => {
                    state.user.profile_image.set(Some(image_id));
                    state.save_profile();
                },
            }
        }));
    }
}

async fn upload_profile_image(file: File) -> Result<ImageId, Box<dyn std::error::Error>> {
    let req = UserImageCreateRequest {
        size: ImageSize::UserProfile,
    };

    let image_id = endpoints::image::user::Create::api_with_auth(Some(req))
        .await
        .map_err(|_err| "Error creating image in db")?
        .id;

    upload_image(image_id, MediaLibrary::User, &file, None)
        .await
        .map_err(|_err| "Error uploading image")?;

    Ok(image_id)
}
