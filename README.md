# Rust journal
A rewrite of my python journal, but in rust. 
It only has the features that I actually use, and the approach to maintaining state is much better, which in turn made the code a lot better.
It has the added bonus that you don't need python to be able to run it.
You can compile it and run it, or just grab the .exe from the releases tab.

## How it works
I wrote this documentation using the journal itself:
```
Help - Saturday 2022/5/21


09:50 am - Before we start:
    09:50 am - Type / help from anywhere to access this text
    09:50 am - Type /exit from anywhere to exit this program

09:51 am - The basics
    09:51 am - Type any text to add an 'entry'
    09:51 am - new entires will be added to the current 'block' of lines
    09:51 am - like
    09:51 am - this

09:52 am - Type dash (-) followed by an entry to start a new block
    09:52 am - Type a (~) on it's own to toggle the last line between being part of a block vs being the start of a new block

09:53 am - (Usefull for when you forget a (-) on the line you just entered

09:53 am - Journal Reading
    09:54 am - Type /prev to view previous entries
    09:54 am - Type /times to view a time breakdown of how much time elapsed between each block.
    09:54 am - Type /gtime to show a more granular (but much harder to read) time breakdown between each entry.

09:54 am - Journal Managing
    09:54 am - You can have multiple journals.
    09:54 am - Type /new to create a new journal. You will be asked to provide a name.
    09:54 am - Type /switch to switch to another journal. This will only work if you have more than one journal.
    09:55 am - Type /find to find some text in the journal. You can use this to go back to an entry by string
```


### Unnecessary info

I tried to do this a few months ago, but I just couldn't wrap my head around rust's type system, particularly the differences between the String and &str and OsString and OsStr classes. But recently, the Rust VS-Code extension seems to have had an update that will intrusively insert the the auto-deduced type into the text, and I believe this helped me understand types a lot better. I used to hate this language, but now that I am able to actually use it, I think it's pretty good. Possibly even the best. Funny how that works, isn't it?
