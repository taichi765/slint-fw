use std::cell::Cell;

use slint::private_unstable_api::re_exports as sp;
use slint_fw as fw;
use slint_fw_macros::slint_fw;

#[slint_fw]
struct InnerTestAdopter {
    count: sp::Property<i32>,
    increment: sp::Callback<(), ()>,
    callback_tracker_increment: sp::Property<()>,
}

struct Adopter(InnerTestAdopter);
