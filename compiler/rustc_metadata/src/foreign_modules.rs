use rustc_data_structures::fx::FxHashMap;
use rustc_hir as hir;
use rustc_hir::def::DefKind;
use rustc_hir::def_id::DefId;
use rustc_middle::query::LocalCrate;
use rustc_middle::ty::TyCtxt;
use rustc_session::cstore::ForeignModule;

pub(crate) fn collect(
    tcx: TyCtxt<'_>,
    LocalCrate: LocalCrate,
) -> FxHashMap<DefId, ForeignModule> {
    let mut modules = FxHashMap::default();

    // We need to collect all the `ForeignMod`, even if they are empty.
    for id in tcx.hir().items() {
        if !matches!(tcx.def_kind(id.owner_id), DefKind::ForeignMod) {
            continue;
        }

        let def_id = id.owner_id.to_def_id();
        let item = tcx.hir().item(id);

        if let hir::ItemKind::ForeignMod { items, .. } = item.kind {
            let foreign_items = items.iter().map(|it| it.id.owner_id.to_def_id()).collect();
            modules.insert(def_id, ForeignModule { foreign_items, def_id });
        }
    }

    modules
}
