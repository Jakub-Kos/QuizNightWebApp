<?php
$pages = [
    'questions'   => 'Questions',
    'main'        => 'Main',
    'menu'        => 'Menu',
    'leaderboard' => 'LeaderBoard',
    'answers'     => 'Answers',
];

// Extract the route from the query string
$page = $_GET['page'] ?? 'main'; // Default to 'main' if no page is specified
// Routing
if (isset($pages[$page])) {
    require "./controllers/" . $pages[$page] . "Controller.php";
    $id = $_GET['id'] ?? null;
    $controllerClass = $pages[$page] . "Controller";
    $controller = new $controllerClass();
}
else{
    http_response_code(404);
    echo $page;
    echo "Help";
    exit;
}
// Dispatching
if (isset($controller)) {
    if (isset($id)) {
        $controller->handle($id); // Pass parameters if available
    } else {
        $controller->handle(); // No parameters needed
    }
}
