<?php
require_once __DIR__ . '/../models/QuizModel.php';
require_once __DIR__ . '/../views/LeaderBoardViewer.php';

class LeaderBoardController{
    private $model;
    private $view;
    public function __construct() {
        $this->view = new LeaderBoardViewer();
        $this->model = new QuizModel();
    }

    public function handle() {
        // Get data from model.
        $questions = $this->model->getLeaderBoard();
        // Render the article using the view
        $this->view->render($questions);
    }
}