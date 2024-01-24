//! Finds cold basic blocks

use rustc_index::IndexVec;
use rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrFlags;
use rustc_middle::mir::*;
use rustc_middle::ty::{self, TyCtxt};
use smallvec::SmallVec;

pub struct FindColdBlocks;

struct Edge {
    from: BasicBlock,
    to: BasicBlock,
}

impl<'tcx> MirPass<'tcx> for FindColdBlocks {
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let local_decls = &body.local_decls;

        // Find initial set of cold blocks
        let mut new = SmallVec::<[BasicBlock; 5]>::new();

        let bbs = body.basic_blocks.as_mut();
        for i in bbs.indices() {
            let terminator = bbs[i].terminator.as_mut().unwrap();
            if let TerminatorKind::Call { func, .. } = &mut terminator.kind
                && let ty::FnDef(def_id, ..) = *func.ty(local_decls, tcx).kind()
            {
                let attrs = tcx.codegen_fn_attrs(def_id);
                if attrs.flags.contains(CodegenFnAttrFlags::COLD) {
                    new.push(i);
                }
            }
        }

        //eprintln!("FindColdBlocks");

        if new.is_empty() {
            return;
        }

        // The preparation of predecessors is done in order to avoid quadratic complexity

        // Find all edges
        let mut edges = Vec::new();
        for i in bbs.indices() {
            let terminator = bbs[i].terminator.as_ref().unwrap();
            for j in terminator.successors() {
                edges.push(Edge { from: i, to: j });
            }
        }

        // a slice with predecessors for each basic block
        let mut pred = IndexVec::<BasicBlock, &[Edge]>::from_elem_n(&[], bbs.len());

        // sort edges by target
        edges.sort_by_key(|e| e.to);
        let mut edges = edges.as_slice();
        // split by target
        while !edges.is_empty() {
            let head = edges.split(|e| e.to != edges[0].to).next().unwrap();
            edges = &edges[head.len()..];

            pred[head[0].to] = head;
        }
        let pred = &pred;

        // update until fixpoint
        while !new.is_empty() {
            //eprintln!("processing cold block");

            let i = new.pop().unwrap();
            if bbs[i].is_cold {
                continue;
            }
            bbs[i].is_cold = true;

            // A new block was just marked cold.
            // Visit all its predecessors and if a predecessor has only cold successors,
            // add the predecessor to the list, so it will be also marked cold.

            for &Edge { from, .. } in pred[i] {
                let mut succ = bbs[from].terminator.as_ref().unwrap().successors();
                if succ.all(|succ| bbs[succ].is_cold) {
                    new.push(from);
                }
            }
        }
    }
}
