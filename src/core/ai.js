const callAnthropic = async (model, prompt, apiKey) => {
  const response = await fetch("https://api.anthropic.com/v1/messages", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "x-api-key": apiKey,
      "anthropic-version": "2023-06-01"
    },
    body: JSON.stringify({
      model: model,
      max_tokens: 4000,
      messages: [{ role: "user", content: prompt }]
    })
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Anthropic API failed: ${response.status} - ${error}`);
  }

  const data = await response.json();
  return data.content[0].text;
};

const callOpenAI = async (model, prompt, apiKey) => {
  const response = await fetch("https://api.openai.com/v1/chat/completions", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Authorization": `Bearer ${apiKey}`
    },
    body: JSON.stringify({
      model: model,
      messages: [{ role: "user", content: prompt }],
      max_tokens: 4000
    })
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`OpenAI API failed: ${response.status} - ${error}`);
  }

  const data = await response.json();
  return data.choices[0].message.content;
};

const callGoogle = async (model, prompt, apiKey) => {
  const response = await fetch(`https://generativelanguage.googleapis.com/v1/models/${model}:generateContent?key=${apiKey}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      contents: [{
        parts: [{ text: prompt }]
      }]
    })
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Google API failed: ${response.status} - ${error}`);
  }

  const data = await response.json();
  return data.candidates[0].content.parts[0].text;
};

const callAI = async (provider, model, prompt, apiKey) => {
  switch (provider) {
    case 'anthropic':
      return await callAnthropic(model, prompt, apiKey);
    case 'openai':
      return await callOpenAI(model, prompt, apiKey);
    case 'google':
      return await callGoogle(model, prompt, apiKey);
    default:
      throw new Error(`Unsupported AI provider: ${provider}`);
  }
};

module.exports = {
  callAI,
};