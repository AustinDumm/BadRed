# TODO

1. Change scripts to be atomic by default
    a. Don't round-robin script execution by default, push_front when you start scripts
    b. Add a RedCall to let a script set itself as non-atomic. Only non-atomic scripts get push_back when yield
    c. Add a Yield RedCall to let an otherwise atomic script be push_back when RedCall::Yield
