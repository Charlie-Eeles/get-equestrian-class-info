This is a little program that aggregates some equestrian class data from Wellington International to make it easier to view.\
It's built using Rust and Cargo, so to use it, make use of the Cargo run/build commands; particularly ```cargo build --release``` to get a working binary.\
It's been hard-coded for a specific trainer (as it was built to help a friend with a very specific problem) if you'd like to change the trainer this works for then you can change the ```pid``` (which atm is 8778) to whatever pid represents the trainer you want to see on wellingtoninternational.com.

---

Running this program can do two things:
- print a prettified table of the data to the console\
    this operation is performed when the 'view' command line argument is passed, like ```cargo run -- view```
- create a csv file of the data under 'wellington_class_data.csv'\
    this is the default operation and will be performed if there are no command line arguments, but can be done alongside the print with the 'create' argument, like ```cargo run -- view create```
