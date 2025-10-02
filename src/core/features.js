const fs = require('fs-extra');
const path = require('path');
const { FEATURES_DIR } = require('./config');

const loadFeature = async (id) => {
  const featurePath = path.join(FEATURES_DIR, `${id}.json`);
  return await fs.readJson(featurePath);
};

const saveFeature = async (feature) => {
  const featurePath = path.join(FEATURES_DIR, `${feature.id}.json`);
  await fs.writeJson(featurePath, feature, { spaces: 2 });
};

const listFeatures = async () => {
  const files = await fs.readdir(FEATURES_DIR);
  const features = [];
  for (const file of files) {
    if (file.endsWith('.json')) {
      const feature = await fs.readJson(path.join(FEATURES_DIR, file));
      features.push(feature);
    }
  }
  return features.sort((a, b) => new Date(b.createdAt) - new Date(a.createdAt));
};

module.exports = {
  loadFeature,
  saveFeature,
  listFeatures,
};