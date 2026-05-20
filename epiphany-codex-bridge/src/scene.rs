use epiphany_core::EpiphanyScene;
use epiphany_core::EpiphanySceneInput;
use epiphany_core::derive_scene;
use epiphany_state_model::EpiphanyThreadState;

pub fn derive_epiphany_scene(
    state: Option<&EpiphanyThreadState>,
    loaded: bool,
    reorient_binding_id: &str,
) -> EpiphanyScene {
    derive_scene(EpiphanySceneInput {
        state,
        loaded,
        reorient_binding_id,
    })
}
