<?php
require_once __DIR__ . '/../models/ArticleModel.php';
require_once __DIR__ . '/../views/ArticleListViewer.php';

class ArticleListController{
    private $model;
    private $view;
    public function __construct() {
        $this->view = new ArticleListViewer();
        $this->model = new ArticleModel();
    }

    public function handle() {
        // Check for POST request to handle creation
        if ($_SERVER['REQUEST_METHOD'] === 'POST') {
            $this->handlePostRequest();
            return;
        }

        if ($_SERVER['REQUEST_METHOD'] === 'DELETE') {
            $this->handleDeleteRequest();
            return;
        }

        // Get data from model.
        $articles = $this->model->getArticleList();

        // Render the article using the view
        $this->view->render($articles);
    }

    private function handlePostRequest() {
        // Validate
        $name = filter_input(INPUT_POST, 'name');
        // Check for required fields
        if (!$name || strlen($name) > 32) {
            http_response_code(400); // Bad request
            echo "Invalid input.";
            return;
        }
        // Add new article to the database
        $retrunedId = $this->model->createArticle($name);

        if ($retrunedId) {
            // Redirect
            header("Location: ./article-edit/$retrunedId");
            exit;
        } else {
            http_response_code(500); // Internal Server Error
            echo "Failed to create the article.";
        }
    }

    private function handleDeleteRequest() {
        // Parse the request body to get the ID (for DELETE, PHP doesn't automatically parse it)
        parse_str(file_get_contents("php://input"), $requestData);

        // Validate the article ID
        if (!isset($requestData['id'])) {
            http_response_code(400); // Bad Request
            echo json_encode(['success' => false, 'message' => 'Invalid or missing article ID.']);
            exit;
        }

        // Attempt to delete the article
        if ($this->model->deleteArticle($requestData['id'])) {
            http_response_code(200); // Success
            echo json_encode(['success' => true, 'message' => 'Article deleted successfully.']);
        } else {
            http_response_code(500); // Internal Server Error
            echo json_encode(['success' => false, 'message' => 'Failed to delete the article.']);
        }
        exit;
    }
}