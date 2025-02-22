<?php
require_once __DIR__ . '/../models/QuizModel.php';
require_once __DIR__ . '/../views/MainViewer.php';

class MainController{
    private $model;
    private $view;
    public function __construct() {
        $this->view = new MainViewer();
        $this->model = new QuizModel();
    }

    public function handle() {
        $this->view->render();
    }
}