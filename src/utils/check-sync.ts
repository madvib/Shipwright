import * as project from '../core/project';
import fs from 'fs-extra';
import path from 'path';

async function checkSync() {
    try {
        const issues = await project.listIssues();
        const inProgress = issues.filter(i => i.status === 'in-progress');

        if (inProgress.length === 0) {
            console.log('✅ No issues currently in-progress. Project stays clean!');
            return;
        }

        console.log(`🔍 Checking ${inProgress.length} in-progress issues...`);

        for (const issue of inProgress) {
            const projectDir = await project.getProjectDir();
            const filePath = path.join(projectDir, 'Issues', 'in-progress', issue.file);
            const content = await fs.readFile(filePath, 'utf8');

            const tasksMatch = content.match(/- \[ \]/g);
            if (!tasksMatch) {
                console.warn(`⚠️ Warning: ${issue.file} is in-progress but has no remaining tasks. Should it be moved to done?`);
            } else {
                console.log(`ℹ️ ${issue.file} has ${tasksMatch.length} tasks remaining.`);
            }
        }
    } catch (err: any) {
        console.error('Error during sync check:', err.message);
    }
}

checkSync();
