#!/usr/bin/env node
/**
 * Move Function Call Graph Generator
 * 
 * This script generates call graphs from Move function analysis results.
 * It can output in various formats including JSON, DOT (Graphviz), and Mermaid.
 * 
 * Usage: node call_graph.js <project_path> [options]
 */

const { spawn } = require('child_process');
const fs = require('fs').promises;
const path = require('path');

class CallGraphGenerator {
    constructor(analyzerBinary = 'move-function-analyzer') {
        this.analyzerBinary = analyzerBinary;
        this.callGraph = {
            nodes: [],
            edges: []
        };
        this.nodeSet = new Set();
    }

    /**
     * Analyze a single function using the Move Function Analyzer
     */
    async analyzeFunction(projectPath, functionName) {
        return new Promise((resolve, reject) => {
            const analyzer = spawn(this.analyzerBinary, [
                '--project-path', projectPath,
                '--function', functionName,
                '--quiet'
            ]);

            let output = '';
            let error = '';

            analyzer.stdout.on('data', (data) => {
                output += data.toString();
            });

            analyzer.stderr.on('data', (data) => {
                error += data.toString();
            });

            analyzer.on('close', (code) => {
                if (code === 0) {
                    try {
                        const results = JSON.parse(output);
                        resolve(results);
                    } catch (e) {
                        console.warn(`Failed to parse JSON for ${functionName}: ${e.message}`);
                        resolve([]); // Return empty array if parsing fails
                    }
                } else {
                    console.warn(`Analysis failed for ${functionName}: ${error}`);
                    resolve([]); // Return empty array on error
                }
            });

            analyzer.on('error', (err) => {
                console.warn(`Failed to start analyzer for ${functionName}: ${err.message}`);
                resolve([]);
            });
        });
    }

    /**
     * Generate call graph from multiple functions
     */
    async generateCallGraph(projectPath, functions) {
        console.error(`Generating call graph for ${functions.length} functions...`);
        
        for (const funcName of functions) {
            console.error(`Analyzing ${funcName}...`);
            const results = await this.analyzeFunction(projectPath, funcName);
            
            for (const result of results) {
                this.addFunctionToGraph(result, funcName);
            }
        }
        
        return this.callGraph;
    }

    /**
     * Add a function and its calls to the call graph
     */
    addFunctionToGraph(result, originalFuncName) {
        const nodeId = `${result.contract}::${originalFuncName}`;
        
        // Add the main function node if not already present
        if (!this.nodeSet.has(nodeId)) {
            this.callGraph.nodes.push({
                id: nodeId,
                label: originalFuncName,
                fullSignature: result.function,
                module: result.contract,
                parameterCount: result.parameters.length,
                lineCount: result.location.end_line - result.location.start_line + 1,
                filePath: result.location.file,
                startLine: result.location.start_line,
                endLine: result.location.end_line,
                type: 'analyzed_function'
            });
            this.nodeSet.add(nodeId);
        }
        
        // Add edges for function calls
        for (const call of result.calls) {
            const callFuncName = this.extractFunctionName(call.function);
            const targetId = `${call.module}::${callFuncName}`;
            
            // Add target node if not present (as external function)
            if (!this.nodeSet.has(targetId)) {
                this.callGraph.nodes.push({
                    id: targetId,
                    label: callFuncName,
                    fullSignature: call.function,
                    module: call.module,
                    filePath: call.file,
                    type: 'external_function'
                });
                this.nodeSet.add(targetId);
            }
            
            // Add edge
            this.callGraph.edges.push({
                source: nodeId,
                target: targetId,
                type: 'calls'
            });
        }
    }

    /**
     * Extract function name from full signature
     */
    extractFunctionName(signature) {
        const match = signature.match(/^([^(]+)/);
        return match ? match[1].trim() : signature;
    }

    /**
     * Export call graph as JSON
     */
    exportAsJSON(pretty = true) {
        return JSON.stringify(this.callGraph, null, pretty ? 2 : 0);
    }

    /**
     * Export call graph as DOT format for Graphviz
     */
    exportAsDOT() {
        const lines = [];
        lines.push('digraph CallGraph {');
        lines.push('  rankdir=TB;');
        lines.push('  node [shape=box, style=rounded];');
        lines.push('');
        
        // Add nodes
        for (const node of this.callGraph.nodes) {
            const color = node.type === 'analyzed_function' ? 'lightblue' : 'lightgray';
            const label = `${node.label}\\n(${node.module})`;
            lines.push(`  "${node.id}" [label="${label}", fillcolor=${color}, style=filled];`);
        }
        
        lines.push('');
        
        // Add edges
        for (const edge of this.callGraph.edges) {
            lines.push(`  "${edge.source}" -> "${edge.target}";`);
        }
        
        lines.push('}');
        return lines.join('\n');
    }

    /**
     * Export call graph as Mermaid diagram
     */
    exportAsMermaid() {
        const lines = [];
        lines.push('graph TD');
        
        // Create node mappings for Mermaid (alphanumeric IDs)
        const nodeMap = new Map();
        let nodeCounter = 1;
        
        for (const node of this.callGraph.nodes) {
            const mermaidId = `N${nodeCounter++}`;
            nodeMap.set(node.id, mermaidId);
            
            const style = node.type === 'analyzed_function' ? 
                `${mermaidId}[${node.label}<br/>${node.module}]` :
                `${mermaidId}(${node.label}<br/>${node.module})`;
            lines.push(`  ${style}`);
        }
        
        lines.push('');
        
        // Add edges
        for (const edge of this.callGraph.edges) {
            const sourceId = nodeMap.get(edge.source);
            const targetId = nodeMap.get(edge.target);
            if (sourceId && targetId) {
                lines.push(`  ${sourceId} --> ${targetId}`);
            }
        }
        
        // Add styling
        lines.push('');
        lines.push('  classDef analyzed fill:#e1f5fe,stroke:#01579b,stroke-width:2px;');
        lines.push('  classDef external fill:#f3e5f5,stroke:#4a148c,stroke-width:1px;');
        
        // Apply styles
        for (const node of this.callGraph.nodes) {
            const mermaidId = nodeMap.get(node.id);
            if (mermaidId) {
                const className = node.type === 'analyzed_function' ? 'analyzed' : 'external';
                lines.push(`  class ${mermaidId} ${className};`);
            }
        }
        
        return lines.join('\n');
    }

    /**
     * Generate statistics about the call graph
     */
    generateStatistics() {
        const analyzedFunctions = this.callGraph.nodes.filter(n => n.type === 'analyzed_function');
        const externalFunctions = this.callGraph.nodes.filter(n => n.type === 'external_function');
        
        const moduleStats = {};
        for (const node of this.callGraph.nodes) {
            if (!moduleStats[node.module]) {
                moduleStats[node.module] = { analyzed: 0, external: 0 };
            }
            moduleStats[node.module][node.type === 'analyzed_function' ? 'analyzed' : 'external']++;
        }
        
        // Calculate complexity metrics
        const complexityStats = analyzedFunctions.map(node => ({
            function: node.label,
            module: node.module,
            outgoingCalls: this.callGraph.edges.filter(e => e.source === node.id).length,
            incomingCalls: this.callGraph.edges.filter(e => e.target === node.id).length,
            parameters: node.parameterCount || 0,
            lines: node.lineCount || 0
        }));
        
        return {
            summary: {
                totalNodes: this.callGraph.nodes.length,
                analyzedFunctions: analyzedFunctions.length,
                externalFunctions: externalFunctions.length,
                totalEdges: this.callGraph.edges.length
            },
            moduleStats,
            complexityStats: complexityStats.sort((a, b) => b.outgoingCalls - a.outgoingCalls)
        };
    }
}

/**
 * Main function
 */
async function main() {
    const args = process.argv.slice(2);
    
    if (args.length === 0) {
        console.error('Usage: node call_graph.js <project_path> [options]');
        console.error('');
        console.error('Options:');
        console.error('  --format <json|dot|mermaid>  Output format (default: json)');
        console.error('  --output <file>              Output file (default: stdout)');
        console.error('  --functions <func1,func2>    Comma-separated list of functions');
        console.error('  --stats                      Include statistics');
        console.error('  --analyzer <path>            Path to move-function-analyzer binary');
        console.error('');
        console.error('Examples:');
        console.error('  node call_graph.js examples/simple-nft');
        console.error('  node call_graph.js examples/simple-nft --format dot --output graph.dot');
        console.error('  node call_graph.js examples/simple-nft --format mermaid --stats');
        process.exit(1);
    }
    
    const projectPath = args[0];
    
    // Parse options
    const options = {
        format: 'json',
        output: null,
        functions: [
            'mint', 'transfer', 'burn', 'mint_and_transfer',
            'create_listing', 'purchase', 'list_nft', 'buy_nft',
            'name', 'description', 'creator', 'price', 'seller',
            'cancel_listing', 'update_description'
        ],
        stats: false,
        analyzer: 'move-function-analyzer'
    };
    
    for (let i = 1; i < args.length; i++) {
        switch (args[i]) {
            case '--format':
                options.format = args[++i];
                break;
            case '--output':
                options.output = args[++i];
                break;
            case '--functions':
                options.functions = args[++i].split(',').map(f => f.trim());
                break;
            case '--stats':
                options.stats = true;
                break;
            case '--analyzer':
                options.analyzer = args[++i];
                break;
        }
    }
    
    // Validate project path
    try {
        await fs.access(projectPath);
        await fs.access(path.join(projectPath, 'Move.toml'));
    } catch (error) {
        console.error(`Error: Invalid project path or missing Move.toml: ${projectPath}`);
        process.exit(1);
    }
    
    try {
        const generator = new CallGraphGenerator(options.analyzer);
        const callGraph = await generator.generateCallGraph(projectPath, options.functions);
        
        let output = '';
        
        // Generate main output
        switch (options.format.toLowerCase()) {
            case 'dot':
                output = generator.exportAsDOT();
                break;
            case 'mermaid':
                output = generator.exportAsMermaid();
                break;
            case 'json':
            default:
                output = generator.exportAsJSON(true);
                break;
        }
        
        // Add statistics if requested
        if (options.stats) {
            const stats = generator.generateStatistics();
            
            if (options.format === 'json') {
                // Merge stats into JSON output
                const jsonData = JSON.parse(output);
                jsonData.statistics = stats;
                output = JSON.stringify(jsonData, null, 2);
            } else {
                // Add stats as comments
                const statsLines = [
                    '',
                    `// Statistics:`,
                    `// Total nodes: ${stats.summary.totalNodes}`,
                    `// Analyzed functions: ${stats.summary.analyzedFunctions}`,
                    `// External functions: ${stats.summary.externalFunctions}`,
                    `// Total edges: ${stats.summary.totalEdges}`,
                    ''
                ];
                output = statsLines.join('\n') + output;
            }
        }
        
        // Output result
        if (options.output) {
            await fs.writeFile(options.output, output);
            console.error(`Call graph saved to ${options.output}`);
            
            if (options.stats) {
                const stats = generator.generateStatistics();
                console.error(`\nStatistics:`);
                console.error(`  Total nodes: ${stats.summary.totalNodes}`);
                console.error(`  Analyzed functions: ${stats.summary.analyzedFunctions}`);
                console.error(`  External functions: ${stats.summary.externalFunctions}`);
                console.error(`  Total edges: ${stats.summary.totalEdges}`);
                
                if (stats.complexityStats.length > 0) {
                    console.error(`\nMost complex functions (by outgoing calls):`);
                    stats.complexityStats.slice(0, 5).forEach(func => {
                        console.error(`  ${func.function}: ${func.outgoingCalls} calls, ${func.parameters} params`);
                    });
                }
            }
        } else {
            console.log(output);
        }
        
    } catch (error) {
        console.error('Error generating call graph:', error.message);
        process.exit(1);
    }
}

// Handle unhandled promise rejections
process.on('unhandledRejection', (reason, promise) => {
    console.error('Unhandled Rejection at:', promise, 'reason:', reason);
    process.exit(1);
});

main();