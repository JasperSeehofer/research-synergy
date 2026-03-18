use gloo_worker::Registrable;
use resyn_worker::ForceLayoutWorker;

fn main() {
    console_error_panic_hook::set_once();
    ForceLayoutWorker::registrar().register();
}
