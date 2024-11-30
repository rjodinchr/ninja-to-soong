use std::collections::HashSet;

use crate::ninja_target::NinjaTarget;
use crate::soong_module::SoongModule;
use crate::soong_package::SoongPackage;
use crate::utils::*;

const CMAKE_GENERATED: &str = "cmake_generated";
const LLVM_PROJECT_NAME: &str = "llvm";
const LLVM_PROJECT_REPO_NAME: &str = "llvm-project";

pub struct LLVM<'a> {
    src_root: &'a str,
    build_root: String,
    ndk_root: &'a str,
}

impl<'a> LLVM<'a> {
    pub fn new(temp_dir: &'a str, ndk_root: &'a str, llvm_project_root: &'a str) -> Self {
        LLVM {
            src_root: llvm_project_root,
            build_root: temp_dir.to_string() + "/" + LLVM_PROJECT_NAME,
            ndk_root,
        }
    }
}

impl<'a> crate::project::Project<'a> for LLVM<'a> {
    fn get_name(&self) -> String {
        LLVM_PROJECT_NAME.to_string()
    }
    fn generate(&self, targets: Vec<NinjaTarget>) -> Result<(), String> {
        let mut package = SoongPackage::new(
            self.src_root,
            self.ndk_root,
            &self.build_root,
            LLVM_PROJECT_REPO_NAME,
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
            self,
        ) {
            return Err(err);
        }
        let mut generated_deps = package.get_generated_deps();
        let include_directories = package.get_include_directories();

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

        remove_directory(add_slash_suffix(self.src_root) + CMAKE_GENERATED)?;
        copy_files(
            generated_deps,
            &self.build_root,
            &(add_slash_suffix(self.src_root) + CMAKE_GENERATED),
        )?;
        touch_directories(&include_directories, &add_slash_suffix(self.src_root))?;

        package.add_module(SoongModule::new_cc_library_headers(
            CC_LIB_HEADERS_LLVM,
            [
                "llvm/include".to_string(),
                CMAKE_GENERATED.to_string() + "/include",
            ]
            .into(),
        ));
        package.add_module(SoongModule::new_cc_library_headers(
            CC_LIB_HEADERS_CLANG,
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

        return package.write(LLVM_PROJECT_REPO_NAME);
    }

    fn get_build_directory(&self) -> Result<String, String> {
        if cmake_configure(
            &(self.src_root.to_string() + "/llvm"),
            &self.build_root,
            self.ndk_root,
            vec![
                LLVM_DISABLE_ZLIB,
                "-DLLVM_ENABLE_PROJECTS=clang;libclc",
                "-DLIBCLC_TARGETS_TO_BUILD=clspv--;clspv64--",
                "-DLLVM_TARGETS_TO_BUILD=",
            ],
        )? {
            cmake_build(
                &self.build_root,
                vec![
                    "clang",
                    "tools/libclc/clspv--.bc",
                    "tools/libclc/clspv64--.bc",
                ],
            )?;
        }
        return Ok(self.build_root.clone());
    }

    fn get_default_cflags(&self) -> HashSet<String> {
        [
            "-Wno-error".to_string(),
            "-Wno-unreachable-code-loop-increment".to_string(),
        ]
        .into()
    }
    fn ignore_target(&self, input: &String) -> bool {
        !input.starts_with("lib")
    }
    fn ignore_define(&self, _define: &str) -> bool {
        true
    }
    fn rework_include(&self, include: &str) -> String {
        include.replace(&self.build_root, CMAKE_GENERATED)
    }
    fn get_headers_to_copy(&self, headers: &HashSet<String>) -> HashSet<String> {
        let mut set = HashSet::new();
        for header in headers {
            set.insert(header.clone());
        }
        return set;
    }
    fn optimize_target_for_size(&self, _target: &String) -> bool {
        true
    }
}
