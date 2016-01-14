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
        echo "let's save some clipboard!" | Clip
        $env:RUST_TEST_THREADS=1
        cargo test
    }
    "build" {
        cargo build
    }
    "doc"   {
        #remove old documentation just in case
        $master_hash = git log -1 --format="%s|%h"
        Remove-Item -Recurse -ErrorAction SilentlyContinue -Force target\doc\
        cargo doc --no-deps
        if ($args[1] -eq "--update-pages") {
            git checkout gh-pages
            Remove-Item -Recurse -ErrorAction SilentlyContinue -Force doc\
            Copy-item -Recurse target\doc\ doc\
            git diff --quiet HEAD
            if ($LASTEXITCODE -eq 1) {
                #commit change in docs
                git add doc/
                git commit -m "Doc-update from $master_hash"
                git push origin HEAD
            }
            git checkout master
        }
    }
    "bot" {
        if ( -Not $env:APPVEYOR) {
            echo "Bot is supposed to run in AppVeyor. Exit..."
            return
        }
        elseif ( $env:TARGET -ne "x86_64-pc-windows-gnu") {
            return
        }

        git config --global credential.helper store
        Add-Content "$env:USERPROFILE\.git-credentials" "https://$($env:git_token):x-oauth-basic@github.com\n"
        git config --global user.name "AppVeyor bot"
        git config --global user.email "douman@gmx.se"
        git config remote.origin.url "https://$($env:git_token)@github.com/DoumanAsh/clipboard-win.git"
        echo "build is done"
        .\build.ps1 doc --update-pages
    }
    default { echo (">>>{0}: Incorrect command" -f $args[0]) }
}
