//! Finds cold basic blocks

use rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrFlags;
use rustc_middle::mir::*;
use rustc_middle::ty::{self, TyCtxt};
use smallvec::SmallVec;

pub struct FindColdBlocks;

impl<'tcx> MirPass<'tcx> for FindColdBlocks {
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let local_decls = &body.local_decls;

        // Find initial set of cold blocks
        let mut unprocessed = SmallVec::<[BasicBlock; 5]>::new();

        let bbs = body.basic_blocks.as_mut();

        for i in bbs.indices() {
            let terminator = bbs[i].terminator.as_mut().unwrap();
            if let TerminatorKind::Call { func, .. } = &mut terminator.kind
                && let ty::FnDef(def_id, ..) = *func.ty(local_decls, tcx).kind()
            {
                let attrs = tcx.codegen_fn_attrs(def_id);
                if attrs.flags.contains(CodegenFnAttrFlags::COLD) {
                    bbs[i].is_cold = true;
                    unprocessed.push(i);
                }
            }
        }

        // Temporary vector to store the successors of a basic block
        // I haven't found a way around the borrow checker without this
        let mut succ = SmallVec::<[BasicBlock; 5]>::new();

        // update until fixpoint
        // worst case complexity:
        //     every block will be visited at most N times,
        //     where N is number of successors
        while !unprocessed.is_empty() {
            let i = unprocessed.pop().unwrap();
            if bbs[i].is_cold {
                continue;
            }

            succ.clear();
            succ.extend(bbs[i].terminator.as_ref().unwrap().successors());

            if succ.len() > 0 && succ.iter().all(|bb| bbs[*bb].is_cold) {
                bbs[i].is_cold = true;
                unprocessed.push(i);
            }
        }
    }
}
