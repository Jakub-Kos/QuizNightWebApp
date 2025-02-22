<?php
$pages = [
    'questions'   => 'Questions',
    'main'        => 'Main',
    'leaderboard' => 'LeaderBoard',
    'answers'     => 'Answers',
];

// Extract the route from the query string
$page = $_GET['page'] ?? 'main'; // Default to 'main' if no page is specified

// Validate the requested page
if (array_key_exists($page, $pages)) {
    $controllerFile = __DIR__ . "/controllers/" . $pages[$page] . "Controller.php";
    
    if (file_exists($controllerFile)) {
        require $controllerFile;
        $controllerClass = $pages[$page] . "Controller";
        
        if (class_exists($controllerClass)) {
            $controller = new $controllerClass();
            $id = $_GET['id'] ?? null;

            // Dispatching: Call the correct method with or without an ID
            if ($id !== null) {
                $controller->handle($id);
            } else {
                $controller->handle();
            }
        } else {
            http_response_code(500);
            echo "Error: Controller class '$controllerClass' not found.";
        }
    } else {
        http_response_code(404);
        echo "Error: Controller file for '$page' not found.";
    }
} else {
    http_response_code(404);
    echo "Error: Page '$page' not recognized.";
}
