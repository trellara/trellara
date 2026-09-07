use crate::rng::DeterministicRng;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Transaction {
    pub(crate) id: String,
    pub(crate) lsn: u64,
}

pub(crate) fn generated_transactions(seed: u64, count: usize) -> Vec<Transaction> {
    let mut rng = DeterministicRng::new(seed);
    let mut lsn = 0x16b6b00;
    (0..count)
        .map(|index| {
            lsn += 1 + rng.next_bounded(4096);
            Transaction {
                id: format!("tx-{:04}-{:016x}", index + 1, rng.next()),
                lsn,
            }
        })
        .collect()
}
