# AI Agents

This document outlines the different AI agents used in this project.

## Overview

Our project utilizes a multi-agent system to automate various aspects of the development and deployment workflow. Each agent is specialized for a specific task, allowing for a more modular and robust system.

## Agents

### 1. Code Generation Agent

*   **Purpose**: Responsible for generating boilerplate code, and implementing new features based on user prompts.
*   **Model**: Google Gemini
*   **Activation**: Triggered by the `generate` command.

### 2. Testing Agent

*   **Purpose**: Automatically generates and runs tests for new and existing code. It ensures code quality and catches regressions.
*   **Model**: Google Gemini
*   **Activation**: Triggered after the Code Generation Agent completes its task, or manually via the `test` command.

### 3. Deployment Agent

*   **Purpose**: Handles the deployment of the application to various environments (staging, production).
*   **Model**: Google Gemini
*   **Activation**: Triggered manually via the `deploy` command.
