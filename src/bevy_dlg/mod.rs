// TODO: Move to separate crate

use anyhow::*;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use dlg::prelude::Dialog;
use std::str::FromStr;

#[derive(TypeUuid, Default)]
#[uuid = "a914688f-8e8e-4584-8eca-8e6322e1de3b"]
pub struct DialogAsset(pub Dialog);

impl AssetLoader for DialogAsset {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let raw = String::from_utf8(Vec::from(bytes))?;

            let dialog = Dialog::from_str(&raw).map_err(|_| anyhow!("can't parse dialog"))?;
            let res = DialogAsset(dialog);

            load_context.set_default_asset(LoadedAsset::new(res));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["dlg"]
    }
}
