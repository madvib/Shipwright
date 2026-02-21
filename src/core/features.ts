import fs from 'fs-extra';
import path from 'path';
import { FEATURES_DIR } from './config';

export interface Feature {
  id: string;
  title: string;
  description: string;
  status: 'draft' | 'in_progress' | 'completed';
  createdAt: string;
  acceptanceCriteria: string[];
  [key: string]: any;
}

export const loadFeature = async (id: string): Promise<Feature> => {
  const featurePath = path.join(FEATURES_DIR, `${id}.json`);
  return await fs.readJson(featurePath);
};

export const saveFeature = async (feature: Feature): Promise<void> => {
  const featurePath = path.join(FEATURES_DIR, `${feature.id}.json`);
  await fs.writeJson(featurePath, feature, { spaces: 2 });
};

export const listFeatures = async (): Promise<Feature[]> => {
  const files = await fs.readdir(FEATURES_DIR);
  const features: Feature[] = [];
  for (const file of files) {
    if (file.endsWith('.json')) {
      const feature = await fs.readJson(path.join(FEATURES_DIR, file));
      features.push(feature);
    }
  }
  return features.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
};