# jamespy

A reimplementation of [jamespy](https://github.com/jamesbt365/jamespy), a spy/utility bot, now in rust using the [poise](https://github.com/serenity-rs/poise) library.

This is actually more usable than the original project, though my scuffed basic knowledge of rust can only go so far, so the code isn't amazing or anything.

## Setup

When you are setting up the bot it is important that you take the badwords.txt, fixwords.txt and the loblist.txt with you and place them next to the binaries for full functionality. So when you set up the bot you need to have these.

If you don't have the badwords and fixwords, you will have problems with messages when the message event is called, and without the loblist the -lob command will not function.

Just run the project or compile it with cargo, and then do what you want with it.

You must have a valid postgresql and redis install set up on your computer, as well as a Discord token, I won't get into these now.

You need to set the following environment variables: JAMESPY_TOKEN, DATABASE_URL and REDIS_URL
