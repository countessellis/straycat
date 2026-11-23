use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use ::straycat::init;
use ::straycat::logger;

///////////// Main

fn main() {
  logger::init();
  init::run();
}
