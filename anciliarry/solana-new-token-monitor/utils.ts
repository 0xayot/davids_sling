import fs from "fs";

export function storeData(dataPath: string, newData: any) {
  fs.readFile(dataPath, (err, fileData) => {
    if (err) {
      console.error(`Error reading file: ${err}`);
      return;
    }
    let json;
    try {
      json = JSON.parse(fileData.toString());
    } catch (parseError) {
      console.error(`Error parsing JSON from file: ${parseError}`);
      return;
    }
    json.push(newData);

    fs.writeFile(dataPath, JSON.stringify(json, null, 2), (writeErr) => {
      if (writeErr) {
        console.error(`Error writing file: ${writeErr}`);
      } else {
        console.log(`New token data stored successfully.`);
      }
    });
  });
}

export async function sendDataToDavidSling(data: any) {
  const url = process.env.DAVID_SLING_URL;
  const authKey = process.env.DAVIDS_POUCH_KEY;
  if (!url || !authKey) {
    console.error(
      "Environment variables DAVID_SLING_URL and X_DAVIDS_POUCH_KEY are required."
    );
    return;
  }
  try {
    const response = await fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "x-davids-pouch-key": authKey,
      },
      body: JSON.stringify(data),
    });
    if (!response.ok) {
      console.error(
        `Error sending data to David Sling: ${response.status} - ${response.statusText}`
      );
    } else {
      console.log("Data sent to David Sling successfully.");
    }
  } catch (error) {
    console.error(`Error sending data to David Sling: ${error}`);
  }
}
