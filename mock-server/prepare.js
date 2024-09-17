// This is a prepare script that is automatically run before "start" command.
//
// It is repsonsible to copy files from sdk-test-data into the proper
import fs from 'node:fs';
import path from 'node:path';
import { exit } from 'node:process';
import { fileURLToPath } from 'node:url';

// The format here is:
// env name -> { api path -> file relative to sdk-test-data }
const envs = {
  ufc: {
    'api/flag-config/v1/config': 'ufc/flags-v1.json',
  },
  obfuscated: {
    'api/flag-config/v1/config': 'ufc/flags-v1-obfuscated.json',
  },
  bandit: {
    'api/flag-config/v1/config': 'ufc/bandit-flags-v1.json',
    'api/flag-config/v1/bandits': 'ufc/bandit-models-v1.json',
  }
}


function main() {
  const __dirname = path.dirname(fileURLToPath(import.meta.url));
  const sdkTestDataPath = path.join(__dirname, '../sdk-test-data/')
  const publicPath = path.join(__dirname, './public/')

  try {
    fs.rmdirSync(path.join(__dirname, 'public'), {recursive: true});
  } catch {
    // ignore if it's not present
  }

  for (const [env, links] of Object.entries(envs)) {
    for (const [target, source] of Object.entries(links)) {
      const sourcePath = path.join(sdkTestDataPath, source);
      const targetPath = path.join(publicPath, env, target);

      fs.mkdirSync(path.dirname(targetPath), { recursive: true });
      console.log(sourcePath, '->', targetPath);
      fs.copyFileSync(sourcePath, targetPath);
    }
  }
}

main()
