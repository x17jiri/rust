//! Finds cold paths in MIR

use rustc_index::IndexVec;
use rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrFlags;
use rustc_middle::mir::*;
use rustc_middle::ty::{self, TyCtxt};

pub struct FindColdPaths;

impl<'tcx> MirPass<'tcx> for FindColdPaths {
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let local_decls = &body.local_decls;

        let mut cold_blocks: IndexVec<BasicBlock, bool> =
            IndexVec::from_elem(false, &body.basic_blocks);

        // Traverse all basic blocks from end of the function to the start.
        for (bb, bb_data) in traversal::postorder(body) {
            let terminator = bb_data.terminator();

            // If a BB ends with a call to a cold function, mark it as cold.
            if let TerminatorKind::Call { ref func, .. } = terminator.kind
                && let ty::FnDef(def_id, ..) = *func.ty(local_decls, tcx).kind()
                && let attrs = tcx.codegen_fn_attrs(def_id)
                && attrs.flags.contains(CodegenFnAttrFlags::COLD)
            {
                cold_blocks[bb] = true;
                continue; // No need to check for other conditions.
            }

            // If a BB has at least one successor and all successors (including the one) are cold,
            // mark this BB as cold.
            let mut succ = terminator.successors();
            if let Some(first) = succ.next()
                && cold_blocks[first]
                && succ.all(|s| cold_blocks[s])
            {
                cold_blocks[bb] = true;
            }
        }

        // Traverse all basic blocks again and fill in cold_targets of SwitchInt terminators.
        // This time the order of traversal is not important.
        let basic_blocks = body.basic_blocks.as_mut_preserves_cfg();
        for bb in basic_blocks.indices() {
            let bb_data = &mut basic_blocks[bb];
            let terminator = bb_data.terminator_mut();
            if let TerminatorKind::SwitchInt { ref mut targets, .. } = terminator.kind {
                // This will mark as cold all arms that lead to cold blocks.
                targets.fill_cold_targets(&cold_blocks);
            }
        }
    }
}
