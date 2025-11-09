#!/usr/bin/env deno run --allow-read

import { ensureDir } from "https://deno.land/std@0.224.0/fs/mod.ts";

function extractImportPath(importLine: string): string | null {
    // Handle @import 'path' or @import "path"
    if (importLine.includes("'")) {
        const start = importLine.indexOf("'");
        const end = importLine.indexOf("'", start + 1);
        if (start !== -1 && end !== -1) {
            return importLine.substring(start + 1, end);
        }
    }

    if (importLine.includes('"')) {
        const start = importLine.indexOf('"');
        const end = importLine.indexOf('"', start + 1);
        if (start !== -1 && end !== -1) {
            return importLine.substring(start + 1, end);
        }
    }

    return null;
}

function inlineCssImports(filePath: string): string {
    const content = Deno.readTextFileSync(filePath);
    let result = "";

    for (const line of content.split('\n')) {
        const trimmed = line.trim();
        if (trimmed.startsWith('@import')) {
            // Parse @import 'path' or @import "path"
            const importPath = extractImportPath(trimmed);
            if (importPath) {
                const baseDir = filePath.substring(0, filePath.lastIndexOf('/'));
                const fullPath = `${baseDir}/${importPath}`;

                try {
                    // Recursively inline imports from the imported file
                    const importedContent = inlineCssImports(fullPath);
                    result += `/* Inlined from ${importPath} */\n`;
                    result += importedContent + '\n';
                } catch (error) {
                    // Keep the original import if file doesn't exist
                    result += line + '\n';
                }
            } else {
                // Keep malformed imports as-is
                result += line + '\n';
            }
        } else {
            // Keep non-import lines as-is
            result += line + '\n';
        }
    }

    return result;
}

async function main() {
    try {
        console.log("Building CSS...");

        // Ensure output directory exists
        await ensureDir('./src/compiled');

        // Process CSS with inlined imports
        const cssContent = inlineCssImports('./web/src/style.css');
        Deno.writeTextFileSync('./src/compiled/stylus.css', cssContent);

        console.log("CSS built successfully!");
        console.log(`Output: ./src/compiled/stylus.css`);
        console.log(`Size: ${cssContent.length} characters`);
    } catch (error) {
        console.error("Error building CSS:", error);
        Deno.exit(1);
    }
}

main();