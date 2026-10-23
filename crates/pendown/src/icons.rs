//! The window's icons: gpui-kit's default set, which its components use,
//! and the toolbar's and help panel's, which aren't in it. Only these are embedded, not the
//! whole Lucide catalogue.

use std::borrow::Cow;

use gpui_kit::{AssetSource, Result, SharedString, assets::Assets};

gpui_kit::assets::icon_assets!(ExtraIcons, [FilePlus, FolderOpen, Save, ImageDown, Play, Pause, StepForward, Footprints, Square, PencilLine]);

pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match ExtraIcons.load(path)? {
            Some(bytes) => Ok(Some(bytes)),
            None => Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = Assets.list(path)?;
        paths.extend(ExtraIcons.list(path)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}
