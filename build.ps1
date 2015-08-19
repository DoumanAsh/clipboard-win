Write-Host "------------------------------"
Write-Host "Clipboard-win build script"
Write-Host "------------------------------"

if ( $args.length -lt 1 ) {
    Write-Host "Usage: build command [options]"
    Write-Host ""
    Write-Host "Commands:"
    Write-Host "    test            Run cargo test without threading."
    Write-Host "    build           Run cargo build."
    Write-Host "    doc             Generate docs."
    Write-Host ""
    Write-Host "Options:"
    Write-Host "    --update_pages  For doc command copy documents to branch gh-pages."
    Write-Host ""
    exit
}

Set-Location $PSScriptRoot #just in case set the location to script(Clipboard-win directory)

switch ($args[0])
{
    "test" {
        $env:RUST_TEST_THREADS=1
        cargo test
    }
    "build" {
        cargo build
    }
    "doc"   {
        #remove old documentation just in case
        Remove-Item -Recurse -ErrorAction SilentlyContinue -Force target\doc\
        cargo doc --no-deps
        if ($args[1] -eq "--update_pages") {
            git checkout gh-pages
            Remove-Item -Recurse -ErrorAction SilentlyContinue -Force doc\
            Copy-item -Recurse target\doc\ doc\
        }
    }
    default { echo (">>>{0}: Incorrect command" -f $args[0]) }
}
