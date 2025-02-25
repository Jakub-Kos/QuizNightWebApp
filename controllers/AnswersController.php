<?php
require_once __DIR__ . '/../models/QuizModel.php';
require_once __DIR__ . '/../views/AnswersViewer.php';

class AnswersController{
    private $model;
    private $view;
    public function __construct() {
        $this->view = new AnswersViewer();
        $this->model = new QuizModel();
    }

    public function handle($round) {
        // Get data from model.
        $questions = $this->model->getQuestionRoundList($round);
        // Render the article using the view
        $this->view->render($questions, null);
    }
}