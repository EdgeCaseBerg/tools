# Duplicate File Monitor

Runs on a folder of your choice and stores the results into a sqlite database.
pops up notifications when a duplicate file is saved so you know. Use dupdb
frontend tool for an easy interface to it.

Setup the dot folder in your home directory (on windows uses your user's folder)
```
mkdir ~/.dupdb
```

When built in debug mode the database will be created next to where the application
is running.