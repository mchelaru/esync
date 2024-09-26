use std::thread;

use crate::semaphore::Semaphore;

/// Process some iterable workload on a given number of threads
///
/// # Examples
///
/// ```
/// # use esync::worker_threads::process;
/// let vec = vec![1, 2, 3, 4, 5];
/// let result = process(vec.iter(), |x| x * x, 2);
/// assert_eq!(1 + 4 + 9 + 16 + 25, result.into_iter().sum());
/// ```
pub fn process<IT, P, R>(it: IT, predicate: P, workers: u32) -> Vec<R>
where
    IT: Iterator,
    IT::Item: Send,
    P: Send + Fn(IT::Item) -> R,
    for<'a> &'a P: Send,
    R: Send,
{
    let mut retval = vec![];
    let sem = Semaphore::new(workers);

    thread::scope(|sc| {
        let mut threads = vec![];
        for s in it {
            sem.wait();
            threads.push(sc.spawn(|| {
                let r = predicate(s);
                sem.release();
                r
            }));
        }
        while let Some(t) = threads.pop() {
            retval.push(t.join().unwrap());
        }
    });

    retval
}

#[cfg(test)]
mod test {
    use crate::worker_threads::process;

    #[test]
    fn process_string() {
        let my_string = "aoigmndeoinaonaoibndf
    dfnwfdbinwefboinwfbisnaina
    ainbainaoinfbf;oainq
    aonaoinaonaib
    aoibnaoibnbaopin
    ainaboaibndsoinasdobnias
    aosibnaosidbnaosbfinab
    aobniaobinasdobina
    aobinasboindofbinosin
    oisndboinapoana
    onsbsoinbfsoidna
    ionfbsoianobian
    odsifnboainabfbd
    bdfosinbaonaoinon
    bsoidnfbsoinabf";

        let s = my_string.split_ascii_whitespace();
        let r = process(s, |p| p.matches("a").count(), 2);
        assert_eq!(42, r.into_iter().reduce(|acc, e| acc + e).unwrap());
    }
}
