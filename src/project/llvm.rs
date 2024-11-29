use std::collections::HashSet;

use crate::filesystem;
use crate::soongmodule::SoongModule;
use crate::soongpackage::SoongPackage;
use crate::target::BuildTarget;
use crate::utils::*;

const CMAKE_GENERATED: &str = "cmake_generated";

pub struct LLVM<'a> {
    src_root: &'a str,
    build_root: &'a str,
    ndk_root: &'a str,
}

impl<'a> LLVM<'a> {
    pub fn new(src_root: &'a str, build_root: &'a str, ndk_root: &'a str) -> Self {
        LLVM {
            src_root,
            build_root,
            ndk_root,
        }
    }
}

impl<'a> crate::project::Project<'a> for LLVM<'a> {
    fn generate(self, targets: Vec<BuildTarget>) -> Result<String, String> {
        let mut package = SoongPackage::new(
            self.src_root,
            self.ndk_root,
            self.build_root,
            "llvm-project_",
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE.TXT",
        );
        if let Err(err) = package.generate(
            vec![
                "libLLVMAggressiveInstCombine.a",
                "libLLVMAnalysis.a",
                "libLLVMAsmParser.a",
                "libLLVMBinaryFormat.a",
                "libLLVMBitReader.a",
                "libLLVMBitWriter.a",
                "libLLVMBitstreamReader.a",
                "libLLVMCFGuard.a",
                "libLLVMCGData.a",
                "libLLVMCodeGenTypes.a",
                "libLLVMCodeGen.a",
                "libLLVMCore.a",
                "libLLVMCoroutines.a",
                "libLLVMCoverage.a",
                "libLLVMDebugInfoBTF.a",
                "libLLVMDebugInfoCodeView.a",
                "libLLVMDebugInfoDWARF.a",
                "libLLVMDebugInfoMSF.a",
                "libLLVMDebugInfoPDB.a",
                "libLLVMDemangle.a",
                "libLLVMExtensions.a",
                "libLLVMFrontendDriver.a",
                "libLLVMFrontendHLSL.a",
                "libLLVMFrontendOffloading.a",
                "libLLVMFrontendOpenMP.a",
                "libLLVMHipStdPar.a",
                "libLLVMIRPrinter.a",
                "libLLVMIRReader.a",
                "libLLVMInstCombine.a",
                "libLLVMInstrumentation.a",
                "libLLVMLTO.a",
                "libLLVMLinker.a",
                "libLLVMMCParser.a",
                "libLLVMMC.a",
                "libLLVMObjCARCOpts.a",
                "libLLVMObject.a",
                "libLLVMOption.a",
                "libLLVMPasses.a",
                "libLLVMProfileData.a",
                "libLLVMRemarks.a",
                "libLLVMSandboxIR.a",
                "libLLVMScalarOpts.a",
                "libLLVMSupport.a",
                "libLLVMSymbolize.a",
                "libLLVMTargetParser.a",
                "libLLVMTarget.a",
                "libLLVMTextAPI.a",
                "libLLVMTransformUtils.a",
                "libLLVMVectorize.a",
                "libLLVMWindowsDriver.a",
                "libLLVMipo.a",
                "libclangAPINotes.a",
                "libclangASTMatchers.a",
                "libclangAST.a",
                "libclangAnalysis.a",
                "libclangBasic.a",
                "libclangCodeGen.a",
                "libclangDriver.a",
                "libclangEdit.a",
                "libclangFrontend.a",
                "libclangLex.a",
                "libclangParse.a",
                "libclangSema.a",
                "libclangSerialization.a",
                "libclangSupport.a",
            ],
            targets,
            &self,
        ) {
            return Err(err);
        }
        let mut generated_deps = package.get_generated_deps();
        let include_directories = package.get_include_directories();

        let dirs_to_remove = vec![CMAKE_GENERATED];
        for dir_to_remove in dirs_to_remove {
            let dir = add_slash_suffix(self.src_root) + dir_to_remove;
            if touch::exists(&dir) {
                if let Err(err) = std::fs::remove_dir_all(&dir) {
                    return error!(format!("remove_dir_all failed: {err}"));
                }
            }
        }

        let missing_generated_deps = vec![
            "include/llvm/Config/llvm-config.h",
            "include/llvm/Config/abi-breaking.h",
            "include/llvm/Config/config.h",
            "include/llvm/Config/Targets.def",
            "include/llvm/Config/AsmPrinters.def",
            "include/llvm/Config/AsmParsers.def",
            "include/llvm/Config/Disassemblers.def",
            "include/llvm/Config/TargetMCAs.def",
            "include/llvm/Support/Extension.def",
            "include/llvm/Support/VCSRevision.h",
            "tools/clang/lib/Basic/VCSVersion.inc",
            "tools/clang/include/clang/Basic/Version.inc",
            "tools/clang/include/clang/Config/config.h",
            "tools/libclc/clspv--.bc",
            "tools/libclc/clspv64--.bc",
        ];
        for header in missing_generated_deps {
            generated_deps.insert(header.to_string());
        }
        match filesystem::copy_files(
            generated_deps,
            &add_slash_suffix(self.build_root),
            &(add_slash_suffix(self.src_root) + &add_slash_suffix(CMAKE_GENERATED)),
        ) {
            Ok(msg) => println!("{msg}"),
            Err(err) => return Err(err),
        }
        match filesystem::touch_directories(&include_directories, &add_slash_suffix(self.src_root))
        {
            Ok(msg) => println!("{msg}"),
            Err(err) => return Err(err),
        }

        package.add_module(SoongModule::new_cc_library_headers(
            LLVM_HEADERS,
            [
                "llvm/include".to_string(),
                CMAKE_GENERATED.to_string() + "/include",
            ]
            .into(),
        ));
        package.add_module(SoongModule::new_cc_library_headers(
            CLANG_HEADERS,
            [
                "clang/include".to_string(),
                CMAKE_GENERATED.to_string() + "/tools/clang/include",
            ]
            .into(),
        ));

        // for clspv
        let opencl_c_base = "clang/lib/Headers/opencl-c-base.h";
        package.add_module(SoongModule::new_copy_genrule(
            clang_headers_name("clang", opencl_c_base),
            opencl_c_base.to_string(),
            opencl_c_base.rsplit_once("/").unwrap().1.to_string(),
        ));
        let clspv_bc = CMAKE_GENERATED.to_string() + "/tools/libclc/clspv--.bc";
        package.add_module(SoongModule::new_copy_genrule(
            llvm_headers_name(CMAKE_GENERATED, &clspv_bc),
            clspv_bc.clone(),
            clspv_bc.rsplit_once("/").unwrap().1.to_string(),
        ));
        let clspv64_bc = CMAKE_GENERATED.to_string() + "/tools/libclc/clspv64--.bc";
        package.add_module(SoongModule::new_copy_genrule(
            llvm_headers_name(CMAKE_GENERATED, &clspv64_bc),
            clspv64_bc.clone(),
            clspv64_bc.rsplit_once("/").unwrap().1.to_string(),
        ));

        return package.write(self.src_root);
    }
    fn get_default_defines(&self) -> HashSet<String> {
        [
            "-Wno-error".to_string(),
            "-Wno-unreachable-code-loop-increment".to_string(),
        ]
        .into()
    }
    fn ignore_target(&self, input: &String) -> bool {
        !input.starts_with("lib")
    }
    fn rework_include(&self, include: &str) -> String {
        include.replace(self.build_root, CMAKE_GENERATED)
    }
    fn get_headers_to_copy(&self, headers: &HashSet<String>) -> HashSet<String> {
        let mut set = HashSet::new();
        for header in headers {
            set.insert(header.clone());
        }
        return set;
    }
    fn optimize_target_for_size(&self, _: &String) -> bool {
        true
    }
}
