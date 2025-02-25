# QuizNightWebApp

To clone this repo use, you need to initialize and update submodules:
``` 
git clone --recurse-submodules <main-repo-url>
```

Or, if you already cloned it without submodules:
```
git submodule update --init --recursive
```

Before running locally, make sure you have `sheet_config.php` in `./models`.
It will look like this:
```
<?php
$sheet_config = [
  'leaderboard'   => 'link to csv file',
  'questions'   => 'link to csv file',
  'statistics'   => 'link to csv file',
];
```