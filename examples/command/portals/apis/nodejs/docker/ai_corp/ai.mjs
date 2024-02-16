import express from "express";
const app = express();
app.use(express.json());

import { LlamaModel, LlamaContext, LlamaChatSession } from "node-llama-cpp";
const modelPath = "./models/capybarahermes-2.5-mistral-7b.Q6_K.gguf";

const model = new LlamaModel({ modelPath: modelPath });
const context = new LlamaContext({ model });
const session = new LlamaChatSession({ context });

app.get("/status", (req, res) => {
  res.json({ status: "running" });
});

app.post("/query", async (req, res) => {
  try {
    const { query } = req.body; // Extracting query from the request body
    if (!query) {
      return res.status(400).json({ error: "No query provided" });
    }

    const answer = await session.prompt(query); // Using the Llama model to process the query
    res.json({ query, answer }); // Sending the query and answer in the response
  } catch (error) {
    console.error(error);
    res.status(500).json({ error: "Failed to process the query" });
  }
});

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => console.log(`Server is running on http://localhost:${PORT}`));
