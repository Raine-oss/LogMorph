// Plugin Main

package io.github.raine.logmorph;

import org.bukkit.Bukkit;
import org.bukkit.ChatColor;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.command.ConsoleCommandSender;
import org.bukkit.command.TabCompleter;
import org.bukkit.entity.Player;
import org.bukkit.plugin.java.JavaPlugin;

import java.io.File;
import java.io.FileWriter;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;

// Plugin Implementation

public class LogMorphPlugin extends JavaPlugin implements CommandExecutor, TabCompleter {

    // Lifecycle

    @Override
    public void onEnable() {
        boolean loaded = LogMorphBridge.loadNative(getDataFolder());
        if (!loaded) {
            getLogger().warning("Failed to load LogMorph native library. Native analysis will be unavailable.");
        } else {
            getLogger().info("LogMorph native engine initialized successfully.");
        }

        if (getCommand("logmorph") != null) {
            getCommand("logmorph").setExecutor(this);
            getCommand("logmorph").setTabCompleter(this);
        }
    }

    @Override
    public void onDisable() {
        getServer().getScheduler().cancelTasks(this);
        getLogger().info("LogMorph plugin disabled.");
    }

    // Command Handler

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!sender.hasPermission("logmorph.admin")) {
            sender.sendMessage(ChatColor.RED + "You do not have permission to use LogMorph.");
            return true;
        }

        if (!LogMorphBridge.isLoaded()) {
            sender.sendMessage(ChatColor.RED + "LogMorph native engine is not loaded. Check server logs.");
            return true;
        }

        if (args.length > 0) {
            String firstArg = args[0].toLowerCase();
            if (firstArg.equals("help") || firstArg.equals("-h") || firstArg.equals("--help") || firstArg.equals("?")) {
                sendHelp(sender);
                return true;
            }
        }

        File serverRoot = getServer().getWorldContainer();
        File logsDir = new File(serverRoot, "logs");
        if (!logsDir.exists()) {
            logsDir = new File("logs");
        }

        File targetLogFile = new File(logsDir, "latest.log");

        // Archive Mode

        if (args.length > 0 && args[0].equalsIgnoreCase("archive")) {
            if (args.length < 2) {
                sender.sendMessage(ChatColor.RED + "Usage: /lm archive <filename.log.gz>");
                return true;
            }

            String archiveName = args[1];
            File candidate = new File(logsDir, archiveName);
            if (!candidate.exists() && !archiveName.endsWith(".log.gz")) {
                candidate = new File(logsDir, archiveName + ".log.gz");
            }

            if (!candidate.exists()) {
                sender.sendMessage(ChatColor.RED + "Archived log not found: " + archiveName + " in logs/ directory.");
                return true;
            }

            targetLogFile = candidate;
        }

        if (!targetLogFile.exists()) {
            sender.sendMessage(ChatColor.RED + "Target log file not found: " + targetLogFile.getName());
            return true;
        }

        final String finalLogPath = targetLogFile.getAbsolutePath();
        final String displayName = targetLogFile.getName();

        // Export Subcommand

        if (args.length > 0 && args[0].equalsIgnoreCase("export")) {
            sender.sendMessage(ChatColor.GRAY + "Exporting server log analysis to JSON...");

            Bukkit.getScheduler().runTaskAsynchronously(this, () -> {
                try {
                    String json = LogMorphBridge.exportJson(finalLogPath);
                    if (!getDataFolder().exists()) {
                        getDataFolder().mkdirs();
                    }
                    File outFile = new File(getDataFolder(), "report.json");
                    try (FileWriter writer = new FileWriter(outFile)) {
                        writer.write(json);
                    }

                    if (!isEnabled()) {
                        return;
                    }

                    Bukkit.getScheduler().runTask(this, () -> {
                        if (!isEnabled()) return;
                        sender.sendMessage(ChatColor.GREEN + "[LogMorph] Successfully exported report to: " + ChatColor.WHITE + "plugins/LogMorph/report.json");
                    });
                } catch (Throwable t) {
                    if (isEnabled()) {
                        Bukkit.getScheduler().runTask(this, () -> {
                            sender.sendMessage(ChatColor.RED + "[LogMorph] Export failed: " + t.getMessage());
                        });
                    }
                }
            });
            return true;
        }

        boolean summaryOnly = false;
        String pluginFilter = null;

        int startIndex = (args.length > 0 && args[0].equalsIgnoreCase("archive")) ? 2 : 0;

        for (int i = startIndex; i < args.length; i++) {
            String arg = args[i].toLowerCase();
            if (arg.equals("summary") || arg.equals("--summary") || arg.equals("-s")) {
                summaryOnly = true;
            } else if ((arg.equals("plugin") || arg.equals("--plugin") || arg.equals("-p")) && i + 1 < args.length) {
                pluginFilter = args[++i];
            } else {
                sender.sendMessage(ChatColor.RED + "[LogMorph] Unknown argument '" + args[i] + "'. Type '/lm help' to see available commands.");
                return true;
            }
        }

        final boolean finalSummaryOnly = summaryOnly;
        final String finalPluginFilter = pluginFilter;
        final boolean isConsole = sender instanceof ConsoleCommandSender;

        sender.sendMessage(ChatColor.GRAY + "Analyzing " + displayName + " with LogMorph...");

        // Async Execution

        Bukkit.getScheduler().runTaskAsynchronously(this, () -> {
            try {
                String result = LogMorphBridge.analyzeLog(
                    finalLogPath,
                    finalSummaryOnly,
                    finalPluginFilter,
                    false
                );

                if (!isEnabled()) {
                    return;
                }

                Bukkit.getScheduler().runTask(this, () -> {
                    if (!isEnabled()) {
                        return;
                    }
                    if (isConsole) {
                        for (String line : result.split("\r?\n")) {
                            sender.sendMessage(line);
                        }
                    } else {
                        String formatted = formatForChat(result);
                        for (String line : formatted.split("\r?\n")) {
                            if (!line.trim().isEmpty()) {
                                sender.sendMessage(line);
                            }
                        }
                    }
                });
            } catch (Throwable t) {
                if (isEnabled()) {
                    Bukkit.getScheduler().runTask(this, () -> {
                        sender.sendMessage(ChatColor.RED + "An error occurred while analyzing logs: " + t.getMessage());
                    });
                }
            }
        });

        return true;
    }

    // Help Menu

    private void sendHelp(CommandSender sender) {
        sender.sendMessage(ChatColor.DARK_AQUA + "=== LogMorph Plugin Commands ===");
        sender.sendMessage(ChatColor.YELLOW + "/lm" + ChatColor.WHITE + " - Analyze logs/latest.log and display report");
        sender.sendMessage(ChatColor.YELLOW + "/lm summary" + ChatColor.WHITE + " - Display statistical summary tables only");
        sender.sendMessage(ChatColor.YELLOW + "/lm archive <file.log.gz>" + ChatColor.WHITE + " - Decompress and analyze archived logs");
        sender.sendMessage(ChatColor.YELLOW + "/lm plugin <Name>" + ChatColor.WHITE + " - Filter errors caused by a specific plugin");
        sender.sendMessage(ChatColor.YELLOW + "/lm export" + ChatColor.WHITE + " - Export analysis to plugins/LogMorph/report.json");
        sender.sendMessage(ChatColor.YELLOW + "/lm help" + ChatColor.WHITE + " - Display this command help menu");
    }

    // Chat Formatter

    private String formatForChat(String ansi) {
        String stripped = ansi.replaceAll("\u001B\\[[;\\d]*m", "");
        return ChatColor.DARK_AQUA + "[LogMorph]\n" + ChatColor.WHITE + stripped;
    }

    // Tab Completion

    @Override
    public List<String> onTabComplete(CommandSender sender, Command command, String alias, String[] args) {
        if (args.length == 1) {
            List<String> options = Arrays.asList("summary", "plugin", "archive", "export", "help");
            List<String> matches = new ArrayList<>();
            for (String opt : options) {
                if (opt.startsWith(args[0].toLowerCase())) {
                    matches.add(opt);
                }
            }
            return matches;
        }

        if (args.length == 2 && args[0].equalsIgnoreCase("archive")) {
            File serverRoot = getServer().getWorldContainer();
            File logsDir = new File(serverRoot, "logs");
            if (!logsDir.exists()) {
                logsDir = new File("logs");
            }
            if (logsDir.exists() && logsDir.isDirectory()) {
                File[] gzFiles = logsDir.listFiles((dir, name) -> name.endsWith(".log.gz"));
                if (gzFiles != null) {
                    List<String> matches = new ArrayList<>();
                    for (File f : gzFiles) {
                        if (f.getName().toLowerCase().startsWith(args[1].toLowerCase())) {
                            matches.add(f.getName());
                        }
                    }
                    return matches;
                }
            }
        }

        return Collections.emptyList();
    }
}
