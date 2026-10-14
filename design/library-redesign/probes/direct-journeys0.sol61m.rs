//! Direct custody comparison; no kitty protocol or private ownership engine.
use kubeless_work_custody::*;
use futures::{channel::oneshot, stream::{FuturesUnordered, StreamExt}, FutureExt};
struct Native;
impl Types for Native {
    type Key = &'static str; type Target = u64; type Body = ();
    type Input = Box<u8>; type Output = Box<u8>; type Unit = ();
    type Decision = &'static str; type Acceptance = ();
}
fn install(owner: &mut Owner<Native>, key: &'static str, value: u8) -> Claim<Native> {
    let target = owner.admit_bound(key, 0).ok().unwrap();
    let offer = owner.install(OfferInput { targets: vec![OfferTarget::Existing(target)], body: (), input: Box::new(value), slots: 0, references: vec![] }).ok().unwrap();
    owner.begin(offer).unwrap()
}
#[test]
fn concurrent_wait_read_short_returns_under_native_selection() {
    futures::executor::block_on(async {
        let mut owner = Owner::<Native>::new();
        let a = install(&mut owner, "launch", 1); let b = install(&mut owner, "text", 2); let c = install(&mut owner, "focus", 3);
        let (atx, arx) = oneshot::channel(); let (btx, brx) = oneshot::channel();
        let mut work = FuturesUnordered::new();
        let av = owner.take(&a).unwrap(); let bv = owner.take(&b).unwrap(); let cv = owner.take(&c).unwrap();
        work.push(async move { arx.await.unwrap(); (a.run(), av) }.boxed());
        work.push(async move { brx.await.unwrap(); (b.run(), bv) }.boxed());
        work.push(async move { (c.run(), cv) }.boxed());
        let (id, value) = work.next().await.unwrap(); assert_eq!(*value, 3);
        drop(owner.return_output(id, value).ok().unwrap());
        assert_eq!(owner.unaccounted().count(), 1); // receipt loss is harmless
        atx.send(()).unwrap(); btx.send(()).unwrap();
        while let Some((id, value)) = work.next().await { owner.return_output(id, value).ok().unwrap(); }
        assert_eq!(owner.unaccounted().count(), 3);
    });
}
#[test]
fn uncertain_launch_then_separate_check_has_no_mutation_input_to_replay() {
    let mut owner = Owner::<Native>::new();
    let launch = install(&mut owner, "launch", 1);
    drop(owner.take(&launch).unwrap()); // lost native driver after take
    assert!(owner.claim_for_input(launch.run()).is_err());
    let check = install(&mut owner, "ls", 2);
    let output = owner.take(&check).unwrap(); owner.return_output(check.run(), output).ok().unwrap();
    let Lookup::Present(old) = owner.run(launch.run()) else { panic!() };
    assert!(old.returned().is_none());
    assert_eq!(owner.runs().count(), 2);
}
