# affi PowerShell tab completion
# Install: . "$(affi receipt completions powershell)" (or source the file)
# Or: affi receipt completions powershell | Out-String | Invoke-Expression

Register-ArgumentCompleter -Native -CommandName @('affi') -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $elements = $commandAst.CommandElements
    $noun = $elements | Select-Object -Skip 1 -First 1
    $verb = $elements | Select-Object -Skip 2 -First 1

    if ($elements.Count -le 2) {
        # Complete nouns
        @('receipt') | Where-Object { $_ -like "$wordToComplete*" } |
            ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }
    } elseif ($noun -eq 'receipt' -and $elements.Count -le 3) {
        # Complete verbs
        @('emit', 'assemble', 'verify', 'show', 'inspect', 'model', 'conformance', 'replay', 'graph', 'stats', 'diagnose', 'mutate', 'bench', 'completions', 'help-refs') |
            Where-Object { $_ -like "$wordToComplete*" } |
            ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }
    } elseif ($noun -eq 'receipt' -and $verb -eq 'emit') {
        @('--type', '--object', '--payload', '--help') |
            Where-Object { $_ -like "$wordToComplete*" } |
            ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterName', $_) }
    } elseif ($noun -eq 'receipt' -and $verb -eq 'assemble') {
        @('--out', '--help') |
            Where-Object { $_ -like "$wordToComplete*" } |
            ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterName', $_) }
    } elseif ($noun -eq 'receipt' -and $verb -eq 'completions') {
        @('bash', 'zsh', 'fish', 'powershell', 'nushell') |
            Where-Object { $_ -like "$wordToComplete*" } |
            ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }
    } else {
        # Default: complete common flags
        @('--help') |
            Where-Object { $_ -like "$wordToComplete*" } |
            ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterName', $_) }
    }
}
