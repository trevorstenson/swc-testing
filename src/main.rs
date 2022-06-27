use std::env;
use std::io::Write;
// use std::time::Instant;
use std::{path::Path, sync::Arc};
use swc::{config::Options};
use swc_ecmascript::{
    ast::{MemberExpr, MemberProp, Expr, Ident},
    transforms::pass::noop,
    visit::{as_folder, Fold},
    visit::{VisitMut, VisitMutWith},
};
use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
    DUMMY_SP,
};
use string_cache::Atom;
use std::fs::File;
use tikv_jemalloc_ctl::{stats, epoch};


fn my_visitor() -> impl Fold {
    as_folder(MyVisitor)
}

struct MyVisitor;
impl VisitMut for MyVisitor {
    fn visit_mut_member_expr(&mut self, expr: &mut MemberExpr) {
        expr.visit_mut_children_with(self);
        if let Expr::Ident(Ident { ref sym, .. }) = *expr.obj {
            if sym.to_string() == "window" {
                if let MemberProp::Ident(Ident { ref sym, .. }) = expr.prop {
                    if sym.to_string() == "location" {
                        *expr = MemberExpr {
                            span: DUMMY_SP,
                            obj: Box::new(Expr::Ident(Ident {
                                span: DUMMY_SP,
                                sym: Atom::from("window"),
                                optional: false,
                            })),
                            prop: MemberProp::Ident(Ident {
                                span: DUMMY_SP,
                                sym: Atom::from("reprise_location"),
                                optional: false,
                            }),
                        }
                    }
                }
            }
        }
    }
}

fn swc_parse(filename: &str) {
    let cm = Arc::<SourceMap>::default();
    let handler = Arc::new(Handler::with_tty_emitter(
        ColorConfig::Auto,
        true,
        false,
        Some(cm.clone()),
    ));
    let c = swc::Compiler::new(cm.clone());
    let fm = cm
    .load_file(Path::new(filename))
    .expect("failed to load file");
    let result = c.process_js_with_custom_pass(
        fm,
        None,
        &handler,
        &Options::default(),
        |_,_| noop(),
        |_,_| my_visitor(),
    );
    let code = match result {
        Ok(output) => output.code,
        Err(_) => panic!("failed to grab code."),
    };
    
    // write to file
    let mut file = File::create(format!("result-{filename}")).expect("create failed");
    file.write_all(code.as_bytes()).expect("write failed");
}

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let e = epoch::mib().unwrap();
    let allocated = stats::allocated::mib().unwrap();
    let resident = stats::resident::mib().unwrap();
    // let now = Instant::now();
    swc_parse(filename);
    e.advance().unwrap();
    let allocated = allocated.read().unwrap();
    let resident = resident.read().unwrap();
    // let elapsed = now.elapsed();
    println!("==={}===", filename);
    println!("MB allocated: {}\nMB resident: {}", allocated / 1000000, resident / 1000000);
    // println!("Elapsed: {:.2?}\n", elapsed);
}