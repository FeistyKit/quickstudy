# Quickstudy

A simple TUI program to help studying. 

## Running

``` shell
$ git clone https://www.github.com/feistykit/quickstudy
$ cd quickstudy
$ make
$ ./quickstudy <input files>
```

Requires [Rust](https://www.rustup.rs) and [ncurses](https://invisible-island.net/ncurses/announce.html) installed.

## Usage

Quickstudy takes a series of input files that describe the questions that should be asked, and then asks them. There is always one question per line, and a question cannot go over to multiple lines. All questions look like one of the two following:

```
# This is a comment! For now, they are just full-line comments, but soon there'll be ones at the end of lines!

# ^ Look! An empty space! Those are allowed!


This is a question, [this is an answer!]. An answer can look like [this first one | this second one!]!
#                    │                │                            │                               │
#                    │                │                            │                               │
# This is a one-off answer. If the user                            If the user inputs either of these
# inputs it fully, they get it right!                              correctly, they get it right!


# In this kind of question, you can input from a list in any order!

The four seasons are: {1}, {1}, {1}, {1}. The best kind of ice cream is either {2} or {2}; spring, summer, fall, winter; vanilla, lavender, peanut butter
#                      │              │                                         │      │ │                             │ 
#                      │              │         ┌───────────────────────────────┘      │ │                             │ 
#                      │              │         │                             ┌────────┘ │                             │    
#                      │              │         Not all options have to be used.         │                             │ 
#                      │              │                                                  │                             │ 
#                      │              │                                                  This separates the regular que stion from the 
#                      │              │                                                  start of the answers and each answer from the
# The user can input four items from the first list (spring,                             next. However, you can't go back to questions
# summer, fall, winter) in any order here. No repeats, though!                           after putting this in!
```

Quickstudy can also search from as many files as you want to use, so you can just put them all in the command to call it!
